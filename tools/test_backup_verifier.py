"""Fast checks for the manual backup test's independent verifier."""
from pathlib import Path
import tempfile
import unittest
import zipfile

import backup_change_detection_test as backup_test


class BackupVerificationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / "source"
        backup_test.seed(self.source)

    def archive(self):
        path = self.root / "backup.zip"
        with zipfile.ZipFile(path, "w") as archive:
            for name, data in backup_test.tree(self.source).items():
                archive.writestr(name + "/" if data is None else name,
                                 b"" if data is None else data)
        return path

    def test_every_mutation_changes_the_inventory(self):
        previous = backup_test.tree(self.source)
        timestamp = (self.source / "same.txt").stat().st_mtime_ns
        for index in range(1, 16):
            with self.subTest(round=index):
                backup_test.mutate(self.source, index)
                current = backup_test.tree(self.source)
                self.assertNotEqual(previous, current)
                backup_test.compare(backup_test.backup_tree(self.archive(), True), current, "ZIP")
                previous = current
                if index == 1:
                    self.assertEqual(timestamp, (self.source / "same.txt").stat().st_mtime_ns)
                    self.assertEqual(len(current["same.txt"]), len(b"before"))
        self.assertNotIn("subtree", previous)
        self.assertNotIn("empty-renamed", previous)
        self.assertEqual(previous["new/nested/moved.txt"], b"move")
        self.assertIn(".hidden", previous)
        self.assertIn("Grüße mit Leerzeichen.txt", previous)

    def test_missing_extra_and_corrupted_entries_fail_for_both_formats(self):
        expected = backup_test.tree(self.source)
        for zipped in (False, True):
            for change in ("missing", "extra", "corrupted", "empty directory"):
                with self.subTest(zipped=zipped, change=change):
                    path = self.source / "same.txt"
                    if change == "missing":
                        path.unlink()
                    elif change == "extra":
                        (self.source / "unexpected").write_bytes(b"extra")
                    elif change == "corrupted":
                        path.write_bytes(b"broken")
                    else:
                        (self.source / "unexpected").mkdir()
                    actual = backup_test.backup_tree(self.archive() if zipped else self.source, zipped)
                    with self.assertRaises(RuntimeError):
                        backup_test.compare(actual, expected, "round 1")
                    path.write_bytes(b"before")
                    extra = self.source / "unexpected"
                    if extra.is_dir():
                        extra.rmdir()
                    elif extra.exists():
                        extra.unlink()

    def test_invalid_zip_fails(self):
        path = self.root / "invalid.zip"
        path.write_bytes(b"not a ZIP")
        with self.assertRaises(zipfile.BadZipFile):
            backup_test.backup_tree(path, True)

    def test_duplicate_zip_paths_fail(self):
        path = self.root / "duplicate.zip"
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr("duplicate/", b"")
            archive.writestr("duplicate", b"file")
        with self.assertRaisesRegex(RuntimeError, "Duplicate"):
            backup_test.backup_tree(path, True)

    def test_missing_and_extra_backup_records(self):
        target = self.root / "backups"
        target.mkdir()
        manifest = target / "manifest.json"
        job = {"name": "Folder", "zip": False, "target": target, "manifest": manifest}
        self.assertFalse(backup_test.verify_round([job], {}, 0))
        backup_test.write_json(manifest, {"records": [{"name": "one"}, {"name": "two"}]})
        with self.assertRaisesRegex(RuntimeError, "extra backup"):
            backup_test.verify_round([job], {}, 0)

    def test_wait_reports_exited_process_and_timeout(self):
        class Process:
            returncode = 7

            def poll(self):
                return self.returncode

        process = Process()
        with self.assertRaisesRegex(RuntimeError, "exited"):
            backup_test.wait_for(process, lambda: True, 0, "test")
        process.returncode = None
        with self.assertRaisesRegex(RuntimeError, "Timeout"):
            backup_test.wait_for(process, lambda: False, 0, "test")


if __name__ == "__main__":
    unittest.main()

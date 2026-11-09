"""Fast checks for the manual backup test's independent verifier."""
from pathlib import Path
import contextlib
import io
import json
import tempfile
import unittest
from unittest.mock import patch
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


class CleanupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.stdout = io.StringIO()
        self.stderr = io.StringIO()
        for redirect in (contextlib.redirect_stdout(self.stdout), contextlib.redirect_stderr(self.stderr)):
            redirect.__enter__()
            self.addCleanup(redirect.__exit__, None, None, None)

    def fixture(self, name="old"):
        output = self.root / (backup_test.OUTPUT_PREFIX + name)
        output.mkdir()
        backup_test.write_json(output / "report.json", {
            "platform": "linux", "binary": "/test/portbak", "started": "2026-11-09",
            "rounds": [], "passed": True})
        (output / "Folder").mkdir()
        (output / "Folder" / "backup").write_bytes(b"backup")
        return output

    def execute(self, output, arguments, error=None):
        def fake_run(binary, directory, report):
            (directory / "data").write_bytes(b"fixture")
            if error is not None:
                raise error

        with patch.object(backup_test.tempfile, "mkdtemp", return_value=str(output)), \
                patch.object(backup_test, "run", side_effect=fake_run):
            return backup_test.main(arguments)

    def test_default_keeps_all_artifacts(self):
        output = self.fixture()
        self.assertEqual(self.execute(output, []), 0)
        self.assertTrue((output / "data").exists())
        self.assertTrue(json.loads((output / "report.json").read_text())["passed"])

    def test_cleanup_after_success_failure_and_interrupt(self):
        for index, error in enumerate((None, RuntimeError("test failure"), KeyboardInterrupt())):
            with self.subTest(error=type(error).__name__):
                output = self.fixture(str(index))
                self.assertEqual(self.execute(output, ["--cleanup"], error), 0 if error is None else 1)
                self.assertFalse(output.exists())

    def test_cleanup_failure_returns_error_and_preserves_report(self):
        output = self.fixture()
        with patch.object(backup_test.shutil, "rmtree", side_effect=PermissionError("denied")):
            self.assertEqual(self.execute(output, ["--cleanup"]), 1)
        self.assertTrue((output / "report.json").exists())
        self.assertIn(str(output), self.stderr.getvalue())
        self.assertEqual(backup_test.cleanup_existing(self.root), 0)
        self.assertFalse(output.exists())

    def test_cleanup_still_runs_when_report_cannot_be_written(self):
        output = self.fixture()
        (output / "report.json").unlink()
        with patch.object(backup_test, "finish_report", side_effect=PermissionError("denied")):
            self.assertEqual(self.execute(output, ["--cleanup"]), 1)
        self.assertFalse(output.exists())

    def test_old_runs_deleted_and_unrecognized_directories_kept(self):
        old = self.fixture()
        foreign = self.root / "unrelated"
        foreign.mkdir()
        malformed = self.fixture("malformed")
        (malformed / "report.json").write_text("not json")
        missing = self.fixture("missing")
        (missing / "report.json").unlink()
        wrong_shape = self.fixture("wrong-shape")
        (wrong_shape / "report.json").write_text("[]")
        self.assertEqual(backup_test.cleanup_existing(self.root), 0)
        self.assertFalse(old.exists())
        for path in (foreign, malformed, missing, wrong_shape):
            self.assertTrue(path.exists())

    def test_active_script_and_app_are_skipped(self):
        for name in (backup_test.TEST_LOCK, "portbak.lock"):
            with self.subTest(lock=name):
                output = self.fixture(name.replace(".", ""))
                with backup_test.FileLock(output / name):
                    self.assertEqual(backup_test.cleanup_existing(self.root), 0)
                    self.assertTrue((output / "Folder" / "backup").exists())
                self.assertEqual(backup_test.cleanup_existing(self.root), 0)
                self.assertFalse(output.exists())
        self.assertIn("Skipped (active)", self.stdout.getvalue())

    def test_directory_symlink_is_skipped(self):
        target = self.root / "foreign"
        target.mkdir()
        link = self.root / (backup_test.OUTPUT_PREFIX + "link")
        try:
            link.symlink_to(target, target_is_directory=True)
        except OSError:
            self.skipTest("Creating symlinks requires additional privileges on this platform")
        self.assertEqual(backup_test.cleanup_existing(self.root), 0)
        self.assertTrue(link.is_symlink())
        self.assertTrue(target.exists())

    def test_existing_cleanup_reports_delete_errors(self):
        output = self.fixture()
        with patch.object(backup_test.shutil, "rmtree", side_effect=PermissionError("denied")):
            self.assertEqual(backup_test.cleanup_existing(self.root), 1)
        self.assertTrue((output / "report.json").exists())
        self.assertIn(str(output), self.stderr.getvalue())

    def test_cleanup_does_not_follow_nested_symlinks(self):
        output = self.fixture()
        foreign = self.root / "foreign"
        foreign.mkdir()
        contents = foreign / "keep"
        contents.write_bytes(b"keep")
        try:
            (output / "linked").symlink_to(foreign, target_is_directory=True)
        except OSError:
            self.skipTest("Creating symlinks requires additional privileges on this platform")
        self.assertEqual(backup_test.cleanup_existing(self.root), 0)
        self.assertFalse(output.exists())
        self.assertEqual(contents.read_bytes(), b"keep")

    def test_standalone_cleanup_does_not_start_an_app(self):
        output = self.fixture()
        with patch.object(backup_test.tempfile, "gettempdir", return_value=str(self.root)), \
                patch.object(backup_test, "run") as run:
            self.assertEqual(backup_test.main(["--cleanup-existing"]), 0)
            run.assert_not_called()
        self.assertFalse(output.exists())

    def test_invalid_argument_combinations(self):
        for args in (["--cleanup", "--cleanup-existing"], ["app", "--cleanup-existing"]):
            with self.subTest(arguments=args), self.assertRaises(SystemExit) as error:
                backup_test.main(args)
            self.assertEqual(error.exception.code, 2)


if __name__ == "__main__":
    unittest.main()

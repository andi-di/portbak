from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile

from package_release import package


class ReleasePackageTests(unittest.TestCase):
    def test_archives_contain_only_distributable_files_and_preserve_executable_mode(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binaries = root / "target/release"
            binaries.mkdir(parents=True)
            (binaries / "portbak").write_bytes(b"linux binary")
            (binaries / "portbak.exe").write_bytes(b"windows binary")
            (binaries / "portbak.json").write_text("private settings")
            (binaries / "backups").mkdir()
            (binaries / "backups/data.txt").write_text("private backup")
            (root / "LICENSE").write_text("app license")
            fonts = root / "assets/fonts"
            fonts.mkdir(parents=True)
            (fonts / "OFL-Test.txt").write_text("font license")
            expected = {"portbak", "LICENSE", "licenses/OFL-Test.txt", "README.txt"}
            linux = package(root, root / "output", "0.2.0", "linux")
            with tarfile.open(linux) as archive:
                self.assertEqual(set(archive.getnames()), expected)
                self.assertEqual(archive.getmember("portbak").mode, 0o755)
                self.assertEqual(archive.extractfile("portbak").read(), b"linux binary")
            windows = package(root, root / "output", "0.2.0", "windows")
            with zipfile.ZipFile(windows) as archive:
                self.assertEqual(set(archive.namelist()), expected - {"portbak"} | {"portbak.exe"})
                self.assertEqual(archive.read("portbak.exe"), b"windows binary")
                self.assertIn(b"0.2.0", archive.read("README.txt"))

    def test_invalid_version_cannot_escape_output_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            for version in ["../elsewhere", "0.2.0-rc.1", "v0.2.0"]:
                with self.assertRaises(ValueError):
                    package(Path(directory), Path(directory), version, "linux")


if __name__ == "__main__":
    unittest.main()

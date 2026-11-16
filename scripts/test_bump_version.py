"""Check version targets without changing the checkout's version."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent


class VersionTests(unittest.TestCase):
    def test_targets(self):
        for target, expected in (("major", "2.0.0"), ("minor", "1.3.0"),
                                 ("patch", "1.2.4")):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                (root / "scripts").mkdir()
                shutil.copy(ROOT / "Makefile", root)
                shutil.copy(ROOT / "scripts/bump_version.py", root / "scripts")
                manifest = root / "Cargo.toml"
                lock = root / "Cargo.lock"
                manifest.write_text('[package]\nname = "emacette"\nversion = "1.2.3"\n')
                lock.write_text('[[package]]\nname = "emacette"\nversion = "1.2.3"\n')
                subprocess.run(["make", target], cwd=root, check=True)
                self.assertIn(f'version = "{expected}"', manifest.read_text())
                self.assertIn(f'version = "{expected}"', lock.read_text())
                lock.write_text("invalid\n")
                before = manifest.read_text()
                result = subprocess.run(["make", target], cwd=root, capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(manifest.read_text(), before)


if __name__ == "__main__":
    unittest.main()

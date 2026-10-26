import contextlib
import importlib.util
import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('system_ca', Path(__file__).parents[1] / 'scripts/harden-system-ca.py')
ca = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ca)

@unittest.skipUnless(sys.platform == 'darwin', 'macOS system CA fixture required')
class SystemCATest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.original = Path('/etc/ssl/cert.pem').read_text()

    def test_real_bundle_removal_is_valid_and_idempotent(self):
        filtered, removed = ca.filter_bundle(self.original)
        ca.validate_bundle(filtered)
        again, second_removed = ca.filter_bundle(filtered)
        self.assertEqual(filtered, again)
        self.assertEqual(second_removed, [])
        self.assertLessEqual(len(removed), 2)

    def test_unexpected_block_structure_fails_closed(self):
        header = sorted(ca.HEADERS)[0]
        content = f'{header}\n-----BEGIN CERTIFICATE-----\nx\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\ny\n-----END CERTIFICATE-----\n### kept\n-----BEGIN CERTIFICATE-----\nz\n-----END CERTIFICATE-----\n'
        with self.assertRaises(ValueError):
            ca.filter_bundle(content)

    def test_apply_preserves_backup_and_repeated_run_is_safe(self):
        with tempfile.TemporaryDirectory() as folder:
            target, backup = Path(folder) / 'cert.pem', Path(folder) / 'backup.pem'
            target.write_text(self.original)
            with patch.object(ca, 'TARGET', target), patch.object(ca, 'BACKUP', backup), patch.object(ca.os, 'geteuid', return_value=0), patch.object(ca.os, 'chown'), patch.object(sys, 'argv', ['harden-system-ca.py']), contextlib.redirect_stdout(io.StringIO()):
                ca.main()
                if backup.exists():
                    self.assertEqual(backup.read_text(), self.original)
                updated = target.read_text()
                ca.main()
                self.assertEqual(target.read_text(), updated)
                if backup.exists():
                    self.assertEqual(backup.read_text(), self.original)
            ca.validate_bundle(target.read_text())

    def test_unprivileged_apply_does_not_change_files(self):
        if not ca.filter_bundle(self.original)[1]:
            self.skipTest('System bundle already protected')
        with tempfile.TemporaryDirectory() as folder:
            target, backup = Path(folder) / 'cert.pem', Path(folder) / 'backup.pem'
            target.write_text(self.original)
            with patch.object(ca, 'TARGET', target), patch.object(ca, 'BACKUP', backup), patch.object(ca.os, 'geteuid', return_value=501), patch.object(sys, 'argv', ['harden-system-ca.py']), contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):
                    ca.main()
            self.assertEqual(target.read_text(), self.original)
            self.assertFalse(backup.exists())

if __name__ == '__main__':
    unittest.main()

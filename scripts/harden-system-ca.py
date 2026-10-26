#!/usr/bin/env python3
"""Apply Apple's eMudhra mitigation to the macOS CLI CA bundle.

Reference: https://support.apple.com/en-us/149555
Removes both affected trust paths, including Sectigo's cross-signs. This may
also reject legitimate sites using unaffected intermediates under these roots.
No changes are made to Keychain or Apple framework trust settings.
"""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

TARGET = Path('/etc/ssl/cert.pem')
BACKUP = Path('/etc/ssl/cert.pem.before-emudhra-20261026')
HEADERS = {
    '=== /C=IN/OU=emSign PKI/O=eMudhra Technologies Limited/CN=emSign ECC Root CA - G3',
    '=== /C=GB/ST=Greater Manchester/L=Salford/O=Comodo CA Limited/CN=AAA Certificate Services',
}
ROOT_NAMES = ('emSign ECC Root CA - G3', 'AAA Certificate Services')


def filter_bundle(content):
    output, removed, skip = [], [], False
    for line in content.splitlines(keepends=True):
        if line.startswith(('=== ', '### ')):
            skip = line.rstrip('\r\n') in HEADERS
            if skip:
                removed.append(line.strip())
        if not skip:
            output.append(line)
    result = ''.join(output)
    if not result.strip() or '-----BEGIN CERTIFICATE-----' not in result:
        raise ValueError('Refusing to create an empty or invalid certificate bundle')
    if content.count('-----BEGIN CERTIFICATE-----') - result.count('-----BEGIN CERTIFICATE-----') != len(removed):
        raise ValueError('Unexpected certificate block structure; no files changed')
    return result, removed


def validate_bundle(content):
    p7 = subprocess.run(['/usr/bin/openssl', 'crl2pkcs7', '-nocrl', '-certfile', '/dev/stdin'],
                        input=content.encode(), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
    certs = subprocess.run(['/usr/bin/openssl', 'pkcs7', '-print_certs', '-noout'],
                           input=p7.stdout, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True).stdout.decode()
    if any(name in certs for name in ROOT_NAMES):
        raise ValueError('An affected root remains; no files changed')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    if sys.platform != 'darwin':
        parser.error('This mitigation is macOS-specific')
    if TARGET.is_symlink() or not TARGET.is_file():
        raise ValueError('Expected a regular system CA bundle')
    original = TARGET.read_bytes()
    updated, removed = filter_bundle(original.decode())
    validate_bundle(updated)
    if not removed:
        print('Already protected: neither affected root is in the system CLI bundle.')
        return
    print('Remove roots: ' + ', '.join(ROOT_NAMES))
    print(f'Certificate count: {original.count(b"-----BEGIN CERTIFICATE-----")} -> {updated.count("-----BEGIN CERTIFICATE-----")}')
    if args.dry_run:
        print('Dry run: system files were not changed.')
        return
    if os.geteuid() != 0:
        parser.error('Administrator access is required. Run with sudo after reviewing --dry-run.')
    # The backup is exclusive and is never overwritten on a subsequent run.
    if not BACKUP.exists():
        with BACKUP.open('xb') as backup:
            backup.write(original)
        shutil.copystat(TARGET, BACKUP)
    elif BACKUP.is_symlink() or not BACKUP.is_file():
        raise ValueError('Backup path is not a regular file')
    # Stage on the same filesystem, validate again, then atomically replace.
    with tempfile.TemporaryDirectory(prefix='.tunebar-ca-', dir=TARGET.parent) as temp:
        stage = Path(temp) / 'cert.pem'
        stage.write_text(updated)
        os.chmod(stage, 0o644)
        os.chown(stage, 0, TARGET.stat().st_gid)
        validate_bundle(stage.read_text())
        if TARGET.read_bytes() != original:
            raise ValueError('System bundle changed concurrently; no files changed')
        os.replace(stage, TARGET)
    print(f'Updated {TARGET}; backup: {BACKUP}')


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Prepare release signing from CI secrets without logging credentials."""
import argparse
import base64
import datetime
import os
import pathlib
import re
import subprocess
import tempfile

REQUIRED = ('SIGNING_KEYSTORE_BASE64', 'SIGNING_STORE_PASSWORD', 'SIGNING_KEY_ALIAS', 'SIGNING_KEY_PASSWORD')

def properties_value(value):
    result = ''
    for char in value:
        if char in '\\:=#! ': result += '\\' + char
        elif char == '\n': result += '\\n'
        elif char == '\r': result += '\\r'
        elif char == '\t': result += '\\t'
        elif ord(char) > 126 or ord(char) < 32:
            raw = char.encode('utf-16-be')
            result += ''.join('\\u' + raw[i:i+2].hex() for i in range(0, len(raw), 2))
        else: result += char
    return result

def prepare(root):
    missing = [name for name in REQUIRED if not os.environ.get(name)]
    if missing: raise RuntimeError('Missing GitHub Actions secrets: ' + ', '.join(missing))
    try: contents = base64.b64decode(os.environ['SIGNING_KEYSTORE_BASE64'], validate=True)
    except Exception: raise RuntimeError('SIGNING_KEYSTORE_BASE64 is not valid base64') from None
    if not contents: raise RuntimeError('Signing keystore is empty')
    destination = root / 'release-signing.keystore'
    if destination.exists() and destination.read_bytes() != contents:
        raise RuntimeError('Existing release.keystore differs; refusing to replace it')
    with tempfile.NamedTemporaryFile(dir=root, prefix='.signing-', delete=False) as file:
        temporary = pathlib.Path(file.name)
        file.write(contents)
    try:
        check = subprocess.run(['keytool', '-list', '-keystore', str(temporary),
            '-storepass:env', 'SIGNING_STORE_PASSWORD', '-alias', os.environ['SIGNING_KEY_ALIAS']],
            capture_output=True)
        if check.returncode: raise RuntimeError('Keystore password or key alias validation failed')
        temporary.replace(destination)
        destination.chmod(0o600)
    finally:
        temporary.unlink(missing_ok=True)
    signing = 'keystore.path=release-signing.keystore\n' + ''.join(key + '=' + properties_value(os.environ[name]) + '\n' for key, name in (
        ('keystore.password', 'SIGNING_STORE_PASSWORD'), ('key.alias', 'SIGNING_KEY_ALIAS'),
        ('key.password', 'SIGNING_KEY_PASSWORD')))
    settings = root / 'signing.properties'
    if settings.exists() and settings.read_text() != signing:
        raise RuntimeError('Existing signing.properties differs; refusing to replace it')
    settings.write_text(signing)
    settings.chmod(0o600)
    epoch = datetime.datetime(2020, 1, 1, tzinfo=datetime.timezone.utc)
    version = int((datetime.datetime.now(datetime.timezone.utc) - epoch).total_seconds())
    if not 211035 < version < 2100000000: raise RuntimeError('Generated Android version code is out of range')
    local = root / 'local.properties'
    existing = local.read_text() if local.exists() else ''
    existing = re.sub(r'^custom\.version\.code=.*(?:\n|$)', '', existing, flags=re.MULTILINE)
    local.write_text(existing.rstrip('\n') + '\ncustom.version.code=' + str(version) + '\n')
    print('Release signing prepared; versionCode=' + str(version))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--project-dir', type=pathlib.Path, default=pathlib.Path.cwd())
    try: prepare(parser.parse_args().project_dir.resolve())
    except RuntimeError as error:
        print(str(error))
        raise SystemExit(1)

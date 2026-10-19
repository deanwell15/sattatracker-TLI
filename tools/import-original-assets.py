#!/usr/bin/env python3
"""Developer-only import of the pinned original graphics archive. Never run by the app.
Requires Pillow for bounded image decoding. Preserves original file bytes.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import tempfile
import urllib.parse
import urllib.request
import zipfile

URL = 'https://github.com/sattatrapy/sattatrackerUpdate/releases/download/Assets/assets.zip'
SHA256 = '83e55bb0e7d193e43a66fa2304f5eff867360fc10b51ee29b45d74ac708669ab'
MAX_ARCHIVE = 256 * 1024 * 1024
MAX_TOTAL = 512 * 1024 * 1024
MAX_FILE = 4 * 1024 * 1024
ROOT = Path(__file__).resolve().parents[1]


def target_path(name):
    if re.fullmatch(r'icons/(?:[0-9]{1,12}|fe)\.png', name):
        return 'src-tauri/resources/' + name
    if re.fullmatch(r'piganim/pig_(?:[0-9]|[1-9][0-9]|1[01][0-9]|120)\.png', name):
        return 'src/assets/animations/pig/' + PurePosixPath(name).name
    if re.fullmatch(r'piganim/coin/coin_(?:[0-9]|[1-9][0-9]|1[01][0-9]|12[01])\.png', name):
        return 'src/assets/animations/coin/' + PurePosixPath(name).name
    raise ValueError('Unexpected archive path: ' + name)


class SafeRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        parsed = urllib.parse.urlsplit(newurl)
        if parsed.scheme != 'https' or parsed.hostname not in {'github.com', 'release-assets.githubusercontent.com'}:
            raise ValueError('Unexpected download redirect host')
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def download(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise ValueError('Refusing to overwrite an existing download; use --archive')
    opener = urllib.request.build_opener(SafeRedirect())
    request = urllib.request.Request(URL, headers={'User-Agent': 'Deanfinite-developer-asset-import'})
    temporary = path.with_suffix(path.suffix + '.partial')
    try:
        total = 0
        with opener.open(request, timeout=30) as response, temporary.open('xb') as out:
            while chunk := response.read(1024 * 1024):
                total += len(chunk)
                if total > MAX_ARCHIVE:
                    raise ValueError('Archive exceeds download limit')
                out.write(chunk)
        if hashlib.sha256(temporary.read_bytes()).hexdigest() != SHA256:
            raise ValueError('Download SHA-256 mismatch')
        temporary.rename(path)
    finally:
        temporary.unlink(missing_ok=True)


def import_archive(path):
    from PIL import Image
    if path.stat().st_size > MAX_ARCHIVE:
        raise ValueError('Archive exceeds size limit')
    if hashlib.sha256(path.read_bytes()).hexdigest() != SHA256:
        raise ValueError('Archive SHA-256 mismatch; source may have changed')
    entries = []
    with tempfile.TemporaryDirectory(prefix='deanfinite-original-assets-') as temp:
        staging = Path(temp)
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
            if len(infos) > 10000 or sum(info.file_size for info in infos) > MAX_TOTAL:
                raise ValueError('Archive count/unpacked size limit exceeded')
            seen = set()
            for info in infos:
                name = info.filename
                if name.startswith('/') or '\\' in name or ':' in name or any(part in {'.', '..'} for part in name.split('/')):
                    raise ValueError('Unsafe archive path')
                if name.casefold() in seen or stat.S_ISLNK(info.external_attr >> 16):
                    raise ValueError('Duplicate path or symlink')
                seen.add(name.casefold())
                target = target_path(name)  # Strict allowlist: no executables or arbitrary names.
                if info.flag_bits & 1 or info.file_size > MAX_FILE or info.file_size > max(1, info.compress_size) * 200:
                    raise ValueError('Encrypted/oversized archive member')
                data = archive.read(info)  # zipfile verifies the member CRC.
                image = Image.open(io.BytesIO(data))
                if image.format not in {'PNG', 'WEBP'} or not 0 < image.width <= 4096 or not 0 < image.height <= 4096 or getattr(image, 'n_frames', 1) != 1:
                    raise ValueError('Unexpected image format/dimensions/animation: ' + name)
                image.load()
                pixels = hashlib.sha256(image.convert('RGBA').tobytes()).hexdigest()
                item = {'source_path': name, 'path': target, 'bytes': len(data),
                        'sha256': hashlib.sha256(data).hexdigest(), 'pixels_sha256': pixels,
                        'width': image.width, 'height': image.height,
                        'mime': 'image/png' if image.format == 'PNG' else 'image/webp'}
                entries.append(item)
                out = staging / target
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_bytes(data)
        required = {f'piganim/pig_{i}.png' for i in range(120)} | {f'piganim/coin/coin_{i}.png' for i in range(122)}
        if not required <= seen or 'icons/100300.png' not in seen:
            raise ValueError('Required original animation frames/base currency icon missing')
        for item in entries:
            destination = ROOT / item['path']
            if destination.exists() and hashlib.sha256(destination.read_bytes()).hexdigest() != item['sha256']:
                raise ValueError('Refusing to overwrite changed graphics: ' + item['path'])
        for item in entries:
            destination = ROOT / item['path']
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(staging / item['path'], destination)
    entries.sort(key=lambda item: item['path'])
    manifest = {'schema': 1, 'source_url': URL, 'archive_sha256': SHA256,
                'archive_bytes': path.stat().st_size, 'file_count': len(entries),
                'total_bytes': sum(item['bytes'] for item in entries),
                'attribution': 'Original asset archive distributed by sattatrapy; game artwork rights remain with the respective owners. No new MIT grant over third-party artwork is asserted.',
                'files': entries}
    (ROOT / 'assets').mkdir(exist_ok=True)
    (ROOT / 'assets/original-assets.manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    icons = sorted(PurePosixPath(item['path']).stem for item in entries if item['source_path'].startswith('icons/'))
    (ROOT / 'src/assets/original-icons.json').write_text(json.dumps({'revision': SHA256[:16], 'ids': icons}, indent=2) + '\n', encoding='utf-8')
    print(f'Imported {len(entries)} original images, {manifest["total_bytes"]:,} bytes; no re-encoding.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--download', action='store_true', help='Download the pinned archive to --archive (developer only)')
    args = parser.parse_args()
    if args.download:
        download(args.archive)
    import_archive(args.archive)


if __name__ == '__main__':
    main()

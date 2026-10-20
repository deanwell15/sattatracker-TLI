"""Generate Deanfinite's original geometric app icon (requires Pillow).
No source images, fonts, downloads or game artwork are used.
"""
from pathlib import Path
from PIL import Image, ImageDraw

root = Path(__file__).resolve().parents[1]
icons = root / 'src-tauri/icons'
image = Image.new('RGBA', (1024, 1024))
draw = ImageDraw.Draw(image)
draw.rounded_rectangle((32, 32, 992, 992), radius=208, fill='#142235')
draw.rectangle((284, 220, 492, 804), fill='#fbbf24')
draw.pieslice((200, 220, 784, 804), -90, 90, fill='#fbbf24')
draw.rectangle((404, 340, 492, 684), fill='#142235')
draw.pieslice((320, 340, 664, 684), -90, 90, fill='#142235')
draw.ellipse((720, 160, 848, 288), fill='#5eead4')
for name, size in [('32x32.png', 32), ('128x128.png', 128), ('128x128@2x.png', 256), ('icon.png', 1024)]:
    image.resize((size, size), Image.Resampling.LANCZOS).save(icons / name)
image.save(icons / 'icon.ico', sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
image.save(icons / 'icon.icns')
image.resize((128, 128), Image.Resampling.LANCZOS).save(root / 'public/app-icon.png')

#!/usr/bin/env python3
"""
Generate 3D quasi-morphism icon for oh-my-file app
"""

from PIL import Image, ImageDraw
import math

def create_icon(size=256):
    # Create a new image with light background
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Colors
    BLUE = (36, 115, 255)        # #2473ff
    CYAN = (47, 208, 170)         # #2fd0aa
    BG_LIGHT = (247, 250, 255)    # #f7faff
    BG_BORDER = (224, 235, 255)   # #e0ebff
    WHITE = (255, 255, 255)
    SHADOW = (0, 0, 0, 30)

    margin = int(size * 0.03)
    circle_size = size - margin * 2

    # Draw background circle with border
    circle_x = margin
    circle_y = margin
    circle_end_x = size - margin
    circle_end_y = size - margin

    # Draw shadow (drop shadow effect)
    shadow_offset = 4
    draw.ellipse(
        [circle_x + shadow_offset, circle_y + shadow_offset,
         circle_end_x + shadow_offset, circle_end_y + shadow_offset],
        fill=(*[int(v * 0.8) for v in (0, 0, 0)], 20)
    )

    # Draw main circle background
    draw.ellipse([circle_x, circle_y, circle_end_x, circle_end_y], fill=BG_LIGHT)

    # Draw border
    draw.ellipse([circle_x, circle_y, circle_end_x, circle_end_y],
                 outline=BG_BORDER, width=2)

    # Draw inner shadow (subtle)
    draw.ellipse([circle_x + 2, circle_y + 2, circle_end_x - 2, circle_end_y - 2],
                 fill=(*[0, 0, 0], 10))

    # Draw arrow 1 (blue, top-left)
    arrow1_x = int(size * 0.2)
    arrow1_y = int(size * 0.3)
    arrow1_width = int(size * 0.2)
    arrow1_height = int(size * 0.15)

    # Arrow body
    draw.rectangle(
        [arrow1_x, arrow1_y, arrow1_x + arrow1_width, arrow1_y + arrow1_height],
        fill=BLUE
    )

    # Arrow head (triangle)
    arrow_head_size = int(size * 0.08)
    arrow_head_x = arrow1_x + arrow1_width
    arrow_head_y = arrow1_y + int(arrow1_height / 2)
    points = [
        (arrow_head_x, arrow_head_y - arrow_head_size),
        (arrow_head_x + arrow_head_size, arrow_head_y),
        (arrow_head_x, arrow_head_y + arrow_head_size)
    ]
    draw.polygon(points, fill=BLUE)

    # Highlight on arrow 1 (white with transparency)
    highlight_height = int(arrow1_height * 0.5)
    draw.rectangle(
        [arrow1_x + 2, arrow1_y + 2,
         arrow1_x + arrow1_width - 2, arrow1_y + highlight_height],
        fill=(*WHITE, 150)
    )

    # Draw arrow 2 (cyan, bottom-right)
    arrow2_x = int(size * 0.55)
    arrow2_y = int(size * 0.55)
    arrow2_width = int(size * 0.2)
    arrow2_height = int(size * 0.15)

    # Arrow body
    draw.rectangle(
        [arrow2_x, arrow2_y, arrow2_x + arrow2_width, arrow2_y + arrow2_height],
        fill=CYAN
    )

    # Arrow head (reversed triangle pointing left-down)
    arrow_head_x = arrow2_x - arrow_head_size
    arrow_head_y = arrow2_y + int(arrow2_height / 2)
    points = [
        (arrow2_x, arrow_head_y - arrow_head_size),
        (arrow_head_x, arrow_head_y),
        (arrow2_x, arrow_head_y + arrow_head_size)
    ]
    draw.polygon(points, fill=CYAN)

    # Shadow under arrow 2 (dark)
    shadow_y = arrow2_y + arrow2_height + 2
    draw.rectangle(
        [arrow2_x, shadow_y, arrow2_x + arrow2_width, shadow_y + 3],
        fill=(*[0, 0, 0], 30)
    )

    # Draw center highlight (light effect)
    highlight_center_x = int(size * 0.5)
    highlight_center_y = int(size * 0.35)
    highlight_radius = int(size * 0.15)

    draw.ellipse(
        [highlight_center_x - highlight_radius,
         highlight_center_y - highlight_radius,
         highlight_center_x + highlight_radius,
         highlight_center_y + highlight_radius],
        fill=(*WHITE, 100)
    )

    # Draw outer glow
    glow_color = (*BLUE, 20)
    draw.ellipse([0, 0, size, size], outline=glow_color, width=1)

    return img

# Generate different sizes
sizes = [256, 128, 64, 32]
icons = {}

for size in sizes:
    icon = create_icon(size)
    icons[size] = icon
    # Save PNG
    icon.save(f'D:/personal/code/project/oh-my-file/src-tauri/icons/icon_{size}.png')
    print(f"✓ Generated {size}x{size} icon")

# Convert 256x256 to ICO format with multiple resolutions
# The main icon
main_icon = icons[256]

# Create ICO with multiple resolutions
icon_list = [icons[size] for size in [256, 128, 64, 32]]
main_icon.save(
    'D:/personal/code/project/oh-my-file/src-tauri/icons/icon.ico',
    format='ICO',
    sizes=[(size, size) for size in sizes]
)

print("✓ Generated icon.ico with multiple resolutions")
print("\n✅ Icon generation complete!")
print("Files created:")
for size in sizes:
    print(f"  • icon_{size}.png")
print("  • icon.ico")

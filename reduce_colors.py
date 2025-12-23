"""
Script to reduce colors in PNG images for Turbo game engine compatibility.
Converts images to indexed PNG with 256 colors max, preserving transparency.
"""

from PIL import Image
import os

def reduce_colors(input_path, output_path=None, max_colors=256):
    """
    Reduce the number of colors in an image to make it compatible with Turbo.
    
    Args:
        input_path: Path to the input image
        output_path: Path for output (if None, overwrites input)
        max_colors: Maximum number of colors (default 256)
    """
    if output_path is None:
        output_path = input_path
    
    # Open the image
    img = Image.open(input_path)
    print(f"Original mode: {img.mode}, Size: {img.size}")
    
    # Check if image has transparency
    has_alpha = img.mode in ('RGBA', 'LA') or (img.mode == 'P' and 'transparency' in img.info)
    
    if has_alpha:
        # Convert to RGBA first if not already
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Quantize with alpha preservation
        # Method 2 (MEDIANCUT) works well for color reduction
        img_quantized = img.quantize(colors=max_colors, method=Image.Quantize.MEDIANCUT)
        
        # Convert back to RGBA to preserve transparency properly
        img_result = img_quantized.convert('RGBA')
        
        # Save as PNG with transparency
        img_result.save(output_path, 'PNG', optimize=True)
    else:
        # No transparency - simple quantization
        if img.mode != 'RGB':
            img = img.convert('RGB')
        
        # Quantize to palette mode (indexed)
        img_quantized = img.quantize(colors=max_colors, method=Image.Quantize.MEDIANCUT)
        
        # Save as indexed PNG
        img_quantized.save(output_path, 'PNG', optimize=True)
    
    # Verify the result
    result_img = Image.open(output_path)
    print(f"Output mode: {result_img.mode}, Size: {result_img.size}")
    print(f"Saved to: {output_path}")
    
    return output_path


def process_sprites_folder(sprites_folder, file_list=None):
    """
    Process multiple sprites in the Sprites folder.
    
    Args:
        sprites_folder: Path to Sprites folder
        file_list: List of specific files to process (if None, process all PNGs)
    """
    if file_list is None:
        # Process all PNG files
        file_list = [f for f in os.listdir(sprites_folder) if f.lower().endswith('.png')]
    
    for filename in file_list:
        filepath = os.path.join(sprites_folder, filename)
        if os.path.exists(filepath):
            print(f"\nProcessing: {filename}")
            try:
                reduce_colors(filepath)
                print(f"✓ Successfully processed {filename}")
            except Exception as e:
                print(f"✗ Error processing {filename}: {e}")
        else:
            print(f"✗ File not found: {filename}")


if __name__ == "__main__":
    # Path to Sprites folder
    sprites_folder = r"c:\Users\Shubh Varshney\Downloads\neuro-santa\gamem\Sprites"
    
    # Files to process - add any files that need color reduction
    files_to_process = [
        "gamewon.png",
        "controls.png",
        # Add more files here if needed:
        # "gameoverpage.png",
        # "time_up.png",
        # "starting_page.png",
    ]
    
    print("=" * 50)
    print("Turbo Game Engine - PNG Color Reducer")
    print("=" * 50)
    
    process_sprites_folder(sprites_folder, files_to_process)
    
    print("\n" + "=" * 50)
    print("Done! Images are now compatible with Turbo.")
    print("=" * 50)

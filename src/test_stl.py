import struct

def read_stl(filepath):
    with open(filepath, 'rb') as f:
        # Check if ASCII
        try:
            content = f.read(80).decode('ascii')
            if content.startswith('solid'):
                print("ASCII STL not supported by this simple script")
                return
        except UnicodeDecodeError:
            pass
        
        f.seek(80)
        num_triangles = struct.unpack('<I', f.read(4))[0]
        min_x, max_x = float('inf'), float('-inf')
        min_y, max_y = float('inf'), float('-inf')
        min_z, max_z = float('inf'), float('-inf')
        
        for i in range(num_triangles):
            f.read(12) # skip normal
            for j in range(3):
                v = struct.unpack('<3f', f.read(12))
                min_x = min(min_x, v[0]); max_x = max(max_x, v[0])
                min_y = min(min_y, v[1]); max_y = max(max_y, v[1])
                min_z = min(min_z, v[2]); max_z = max(max_z, v[2])
            f.read(2) # attr
            
        print(f"X: {min_x} to {max_x}")
        print(f"Y: {min_y} to {max_y}")
        print(f"Z: {min_z} to {max_z}")

try:
    read_stl('../Models/art_gallery.stl')
except Exception as e:
    print(e)

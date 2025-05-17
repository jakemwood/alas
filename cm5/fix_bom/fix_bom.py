import sexpdata
import csv
from decimal import Decimal

# The plug is 3.7mm wide
# The plug is 22.8mm wide

# Load the KiCad PCB file
with open('../CM5IO.kicad_pcb', 'r') as f:
    content = f.read()

# Parse the S-expression
data = sexpdata.loads(content)

# Search function
def find_footprint_at(data, footprint_name):
    for item in data:
        if isinstance(item, list) and item and item[0].value() == 'footprint' and item[1] == footprint_name:
            for param in item:
                if isinstance(param, list) and param[0].value() == 'at':
                    x = param[1]
                    y = param[2]
                    return (Decimal(x), Decimal(y)) 
                # TODO: find rotation
    return None

# Target footprint
footprint_name = "CM5IO:Raspberry-Pi-5-Compute-Module"

coords = find_footprint_at(data, footprint_name)

if coords:
    print(f"Coordinates for footprint '{footprint_name}': {coords}")
else:
    print(f"Footprint '{footprint_name}' not found.")
    exit(1)

rows = []
with open("../jlcpcb/production_files/BOM-CM5IO.csv", "r") as f:
    reader = csv.reader(f)
    for row in reader:
        if row[0] == "ComputeModule5-CM5":
            row[4] = 2  # update to two needed
        rows.append(row)
    
with open("../jlcpcb/production_files/BOM-CM5IO-fixed.csv", "w") as f:
    writer = csv.writer(f)
    writer.writerows(rows)

rows = []
with open("../jlcpcb/production_files/CPL-CM5IO.csv", "r") as f:
    reader = csv.reader(f)
    for row in reader:
        if row[0] == "Module1":
            # need to add two rows for the module

            # Original:
            # Module1,ComputeModule5-CM5,Raspberry-Pi-5-Compute-Module,179.0,-97.5,180.0,top

            # OG dimensions from PCB file:
            # 195.5, 73.5

            # Desired:
            # Module1	ComputeModule5-CM5	Raspberry-Pi-5-Compute-Module	162.0	-95.0	90.0	top
            # Module1	ComputeModule5-CM5	Raspberry-Pi-5-Compute-Module	195.96	-95.0	90.0	top
            original_x, original_y = coords

            # For right-side up rotation
            # y = -(original_y + Decimal("21.5"))  # counterintuitive but the dimensions are weird in these files
            # x = original_x - Decimal("33.5")  # move the first one 17 mm to the left

            # For upside down rotation
            y = -(original_y - Decimal("21.5"))  # counterintuitive but the dimensions are weird in these files
            x = original_x + Decimal("33.5")  # move the first one 17 mm to the left

            row[3] = x
            row[4] = y
            row[5] = 90  # TODO: if the board ever rotates, change this

            rows.append(row)

            row = [x for x in row]

            # For right-side up orientation
            # x = Decimal(original_x) - Decimal("0.46")  # that explains this

            # For upside down orientation
            x = Decimal(original_x) - Decimal("0.46")  # that explains this
            row[3] = x
            rows.append(row)
        else:
            rows.append(row)

with open("../jlcpcb/production_files/CPL-CM5IO-fixed.csv", "w") as f:
    writer = csv.writer(f)
    writer.writerows(rows)

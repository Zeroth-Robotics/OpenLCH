
from openlch import HAL
 
hal = HAL()
 
# Get the current position of the servo
positions = hal.servo.get_positions()
print(positions)









# Set the position of the servos
# hal.servo.set_positions([1, 2, 3], [90, -90, 13.6])
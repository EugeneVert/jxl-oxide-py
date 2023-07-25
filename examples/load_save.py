from PIL import Image
import jxl_oxide  # noqa

print("Decode")
image = Image.open("1.jxl")

print("Size")
print(image.size)

print("Load & Save")
image.save("1.jpg")

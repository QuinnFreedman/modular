from random import random
path = "M 0 100"
target = 100
actual = 100
path2 = "M 0 100"
for i in range(400):
    rand = random()
    if rand > .66:
        path += " h 1"
    elif rand > .33:
        path += " h 1 v 1"
        target += 1
    else:
        path += " h 1 v -1"
        target -= 1
    delta = target - actual
    actual += .05 * delta
    path2 += f" L {i+1} {actual}"

with open("brownian.svg", "w") as f:
    f.write('<svg viewBox="0 0 400 200">\n')
    f.write(f'<path stroke="#e14646ff" fill="none" d="{path}" />\n')
    f.write(f'<path stroke="yellow" fill="none" d="{path2}" />\n')
    f.write('</svg>\n')

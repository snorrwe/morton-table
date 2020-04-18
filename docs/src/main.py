import matplotlib.pyplot as plt
import numpy as np


def morton_code(x, y):
    return partition(x) | (partition(y) << 1)


def partition(n):
    n = (n ^ (n << 8)) & 0x00FF00FF
    n = (n ^ (n << 4)) & 0x0F0F0F0F
    n = (n ^ (n << 2)) & 0x33333333
    return (n ^ (n << 1)) & 0x55555555


fig, ax = plt.subplots()

points = np.array([[(x, y) for y in range(20)] for x in range(20)]).reshape(20 * 20, 2)
points = np.array(sorted(points, key=lambda x: morton_code(x[0], x[1])))

for p1, p2 in zip(points[:-1], points[1:]):
    [x1, y1] = p1
    [x2, y2] = p2
    dx = x2 - x1
    dy = y2 - y1
    ax.arrow(
        x1,
        y1,
        dx=dx,
        dy=dy,
        head_width=0.05,
        head_length=0.1,
        fc="k",
        ec="k",
        color="blue",
    )
    ax.text(
        x1, y1, morton_code(x1, y1), color="red"
    )

x, y = points.T
ax.scatter(x, y, 10, c="red")


plt.show()

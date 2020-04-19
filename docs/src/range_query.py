from pathlib import Path
import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation, writers
from matplotlib.patches import Rectangle
from matplotlib.collections import PatchCollection
import numpy as np

OUTDIR = Path(".") / "out"
LIMX = 32
LIMY = 32


def morton_code(x, y):
    return partition(x) | (partition(y) << 1)


def partition(n):
    n = (n ^ (n << 8)) & 0x00FF00FF
    n = (n ^ (n << 4)) & 0x0F0F0F0F
    n = (n ^ (n << 2)) & 0x33333333
    return (n ^ (n << 1)) & 0x55555555


points = np.array([[(x, y) for y in range(LIMY)] for x in range(LIMX)]).reshape(
    LIMX * LIMY, 2
)
points = np.array(sorted(points, key=lambda x: morton_code(x[0], x[1])))

query = np.array([10, 12, 16, 16]).reshape(2, 2)

visited = np.arange(morton_code(*query[0]), morton_code(*query[1]) + 1)
queried = points[visited]


fig, ax = plt.subplots()
ax.set_xlim(-2, LIMX + 2)
ax.set_ylim(LIMY + 2, -2)
ax.axis("off")

x, y = points.T
ax.scatter(x, y, c="tab:blue", alpha=0.2)

(plot,) = ax.plot([], [], c="red")


def draw_query_rect():
    x, y = query[0]
    w, h = query[1] - query[0]
    queryrect = Rectangle((x - 0.2, y - 0.2), w + 0.4, h + 0.4)

    pc = PatchCollection([queryrect], facecolor="blue", alpha=0.3, edgecolor="yellow",)

    # Add collection to axes
    ax.add_collection(pc)


def update(i):
    x, y = queried.T
    plot.set_data(x[:i], y[:i])
    return (plot,)


draw_query_rect()

anim = FuncAnimation(
    fig, update, frames=np.arange(len(queried) + 1), interval=3, blit=True, repeat=False
)


OUTDIR.mkdir(exist_ok=True)
Writer = writers["ffmpeg"]
writer = Writer(fps=30, metadata=dict(artist="Daniel Kiss"), bitrate=1000)
anim.save(f"{OUTDIR}/range_query.mp4", writer=writer)

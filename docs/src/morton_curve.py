"""
Demonstrates how the Mortron curve fills the space
"""
import sys
from pathlib import Path
import os
import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation, writers
import numpy as np

OUTDIR = Path(".") / "out"

LIMX = 2 ** 5
LIMY = 2 ** 5


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


fig, ax = plt.subplots()


class Anim:
    def __init__(self, ax, points):
        self.ax = ax
        ax.set_xlim(-2, LIMX + 2)
        ax.set_ylim(-2, LIMY + 2)
        ax.grid(True)

        ax.set_xticks(np.arange(LIMX))
        ax.set_yticks(np.arange(LIMY))

        (self.plot,) = ax.plot([], [], c="red")
        self.points = points

    def init(self):
        return (self.plot,)

    def __call__(self, i):
        x, y = self.points.T
        self.plot.set_data(x[:i], y[:i])
        return (self.plot,)


anim = Anim(ax, points)
anim = FuncAnimation(
    fig,
    anim,
    frames=np.arange(len(points)),
    init_func=anim.init,
    interval=30,
    blit=True,
    repeat=False,
)

OUTDIR.mkdir(exist_ok=True)
Writer = writers["ffmpeg"]
writer = Writer(fps=30, metadata=dict(artist="Daniel Kiss"), bitrate=1800)
anim.save(f"{OUTDIR}/morton_curve.mp4", writer=writer)

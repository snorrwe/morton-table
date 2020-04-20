from pathlib import Path
import matplotlib

#  matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation, writers
from matplotlib.patches import Rectangle
from matplotlib.collections import PatchCollection
import numpy as np

OUTDIR = Path(".") / "out"
LIMX = 32
LIMY = 32

DE_BRUIJN_BIT_POS = np.array(
    [
        0,
        9,
        1,
        10,
        13,
        21,
        2,
        29,
        11,
        14,
        16,
        18,
        22,
        25,
        3,
        30,
        8,
        12,
        20,
        28,
        15,
        17,
        24,
        7,
        19,
        27,
        23,
        6,
        26,
        5,
        4,
        31,
    ]
)


def morton_code(x, y):
    return partition(x) | (partition(y) << 1)


def partition(n):
    n = (n ^ (n << 8)) & 0x00FF00FF
    n = (n ^ (n << 4)) & 0x0F0F0F0F
    n = (n ^ (n << 2)) & 0x33333333

    return (n ^ (n << 1)) & 0x55555555


def as_point(morton):
    x = reconstruct(morton)
    y = reconstruct(morton >> 1)

    return np.array([x, y])


def reconstruct(n):
    n &= 0x55555555
    n |= n >> 1
    n &= 0x33333333
    n |= n >> 2
    n &= 0x0F0F0F0F
    n |= n >> 4
    n &= 0x00FF00FF
    n |= n >> 8

    return n & 0x0000FFFF


def litmax_bigmin(mortonmin, minpos, mortonmax, maxpos):
    [x1, y1] = minpos
    [x2, y2] = maxpos
    diff = mortonmin ^ mortonmax
    diff_msb = msb_de_bruijn(diff)

    if diff_msb & 1 == 0:
        x1, x2 = impl_litmax_bigmin(x1, x2, diff_msb // 2)

        return morton_code(x1, y2), morton_code(x2, y1)
    else:
        m1, y2 = impl_litmax_bigmin(y1, y2, diff_msb // 2)
        y1 = m1 | y1

        return morton_code(x2, y1), morton_code(x1, y2)


def impl_litmax_bigmin(a, b, diff_msb):
    prefix2 = 1 << diff_msb
    prefix1 = prefix2 - 1

    # calculate the common most significant bits
    # aka. the prefix
    mask = ~(~prefix2 & prefix1)
    z = (a & b) & mask
    # append the suffixes
    litmax = z | prefix1
    bigmin = z | prefix2

    return litmax, bigmin


def msb_de_bruijn(v):

    # first round down to one less than a power of 2
    v |= v >> 1
    v |= v >> 2
    v |= v >> 4
    v |= v >> 8
    v |= v >> 16

    # *magic*
    ind = v * 0x07C4ACDD
    ind = ind >> 27

    return DE_BRUIJN_BIT_POS[ind]


points = np.array([[(x, y) for y in range(LIMY)] for x in range(LIMX)]).reshape(
    LIMX * LIMY, 2
)
points = np.array(sorted(points, key=lambda x: morton_code(x[0], x[1])))

query = np.array([10, 12, 16, 16]).reshape(2, 2)


def calc_visited(points, query, visited=None, queries=None):
    if visited is None:
        visited = []
    if queries is None:
        queries = []
    queries.append(query)
    min, max = morton_code(*query[0]), morton_code(*query[1])

    if max < min:
        return visited, queries

    if max - min > 16:
        # split
        litmax, bigmin = litmax_bigmin(min, query[0], max, query[1])
        litmax = as_point(litmax)
        bigmin = as_point(bigmin)
        calc_visited(points, (query[0], litmax), visited, queries)

        return calc_visited(points, (bigmin, query[1]), visited, queries)

    indices = np.arange(min, max + 1)
    print(f"Query {query} visiting {len(indices)} points")
    visited.append(points[indices])

    return visited, queries


visited, queries = calc_visited(points, query)
visited = np.array([x for v in visited for x in v])
queries = np.array(queries)

print(f"Visiting {len(visited)} nodes in {len(queries)} sub-queries", queries)


fig, ax = plt.subplots()
ax.set_xlim(-2, LIMX + 2)
ax.set_ylim(LIMY + 2, -2)
ax.axis("off")

x, y = points.T

ax.scatter(x, y, c="tab:blue", alpha=0.2)

(plot,) = ax.plot([], [], c=[1, 0, 0])


def draw_query_rect(query):
    x, y = query[0]
    w, h = query[1] - query[0]
    queryrect = Rectangle((x - 0.2, y - 0.2), w + 0.4, h + 0.4)

    pc = PatchCollection(
        [queryrect], facecolor=[0, 0, 0, 0], edgecolor=[0.8, 0.3, 0, 0.7],
    )

    # Add collection to axes
    ax.add_collection(pc)


def update(i):
    x, y = visited.T
    plot.set_data(x[:i], y[:i])
    return (plot,)


draw_query_rect(queries[0],)

anim = FuncAnimation(
    fig, update, frames=np.arange(len(visited)+1), interval=200, blit=True, repeat=False,
)


#  OUTDIR.mkdir(exist_ok=True)
#  Writer = writers["ffmpeg"]
#  writer = Writer(fps=30, metadata=dict(artist="Daniel Kiss"), bitrate=1000)
#  anim.save(f"{OUTDIR}/range_query.mp4", writer=writer)

plt.show()

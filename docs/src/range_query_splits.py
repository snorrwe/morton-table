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
SPLIT_THRESHOLD = 16

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

    min, max = morton_code(*query[0]), morton_code(*query[1])
    l = max + 1 - min
    queries.append((query, l))

    if l < 0:
        return visited, queries

    if l > SPLIT_THRESHOLD:
        # split
        litmax, bigmin = litmax_bigmin(min, query[0], max, query[1])
        litmax = as_point(litmax)
        bigmin = as_point(bigmin)
        visited, queries = calc_visited(points, (query[0], litmax), visited, queries)

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
PADDING = 6
ax.set_xlim(query[0, 0] - PADDING, query[1, 0] + PADDING)
ax.set_ylim(query[1, 1] + PADDING, query[0, 1] - PADDING)
ax.axis("off")

x, y = points.T

ax.scatter(x, y, c="tab:blue", alpha=0.2)

(plot,) = ax.plot([], [], c=[1, 0, 0.2, 0.7])


def draw_query_rect(queries):
    l = max(5, len(queries))

    for i, q in enumerate(queries):
        x, y = q[0]
        w, h = q[1] - q[0]
        queryrect = Rectangle((x - 0.2, y - 0.2), w + 0.4, h + 0.4)
        pc = PatchCollection(
            [queryrect],
            facecolor=[0, 0, 0, 0],
            edgecolor=[1 / (l - i), 0.8, 1 / (i + 1), 0.7],
            linewidth=2,
        )
        ax.add_collection(pc)
    return ax


qs = queries[:, 1] <= SPLIT_THRESHOLD
qs = queries[qs]
qs = qs[qs[:, 1] > 0]
batcheslen = qs[:, 1]

assert sum(batcheslen) == len(visited)


def update(i):
    batchind = 0
    batchsum = 0

    for b in batcheslen:
        batchsum += b

        if batchsum > i:
            break
        batchind += 1

    batchind = min(batchind, len(qs) - 1)

    x, y = visited.T
    plot.set_data(x[:i], y[:i])
    draw_query_rect(qs[: batchind + 1, 0])

    return (plot, )


q = queries[0, 0]
x, y = q[0]
w, h = q[1] - q[0]
queryrect = Rectangle((x - 0.2, y - 0.2), w + 0.4, h + 0.4)
pc = PatchCollection(
    [queryrect], facecolor=[0, 0, 0, 0], edgecolor=[1, 0.3, 0.3, 0.3], linewidth=2,
)
ax.add_collection(pc)

anim = FuncAnimation(
    fig,
    update,
    frames=np.arange(len(visited) + 1),
    interval=500,
    blit=True,
    repeat=False,
)


OUTDIR.mkdir(exist_ok=True)
Writer = writers["ffmpeg"]
writer = Writer(fps=10, metadata=dict(artist="Daniel Kiss"), bitrate=1000)
anim.save(f"{OUTDIR}/range_query_splits.mp4", writer=writer)

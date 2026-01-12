# Squad Structure Guide

This document describes the hierarchical organization of AI agents for Project Zang.

## Organization Chart

```
                    +-----------+
                    |   Human   |
                    | (README)  |
                    +-----+-----+
                          |
                          | Sets "Project Direction"
                          v
                    +-----------+
                    | Director  |
                    | (Window 1)|
                    +-----+-----+
                          |
          +---------------+---------------+
          |                               |
          v                               v
    +------------+                  +------------+
    |  EM-Forge  |                  |  EM-Anvil  |
    | (Window 2) |                  | (Window 3) |
    +-----+------+                  +-----+------+
          |                               |
    +--+--+--+--+--+               +--+--+--+--+--+
    |  |  |  |  |  |               |  |  |  |  |  |
    v  v  v  v  v  v               v  v  v  v  v  v
   W1 W2 W3 W4 W5                 W1 W2 W3 W4 W5
```

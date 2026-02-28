pub const EMPTY_GRAPH: [[usize; 0]; 0] = [];

pub const PAIR_GRAPH: [[usize; 1]; 2] = [[1], [0]];

pub const TRIANGLE_GRAPH: [[usize; 2]; 3] = [[1, 2], [0, 2], [0, 1]];

pub const K4_GRAPH: [[usize; 3]; 4] = [[1, 2, 3], [0, 2, 3], [0, 1, 3], [0, 1, 2]];

pub const C4_GRAPH: [[usize; 2]; 4] = [[1, 3], [0, 2], [1, 3], [0, 2]];

pub const K5_GRAPH: [[usize; 4]; 5] = [[1, 2, 3, 4], [0, 2, 3, 4], [0, 1, 3, 4], [0, 1, 2, 4], [0, 1, 2, 3]];

pub const C5_GRAPH: [[usize; 2]; 5] = [[1, 4], [0, 2], [1, 3], [2, 4], [0, 3]];

pub const PENTAGON_GRAPH: [[usize; 2]; 5] = [[1, 4], [0, 2], [1, 3], [2, 4], [0, 3]];

pub const PENTAGRAM_GRAPH: [[usize; 2]; 5] = [[2, 3], [3, 4], [4, 0], [0, 1], [1, 2]];

pub const PETERSEN_GRAPH: [[usize; 3]; 10] =
    [
        [1, 4, 5],
        [0, 2, 6],
        [1, 3, 7],
        [2, 4, 8],
        [0, 3, 9],
        [0, 7, 8],
        [1, 8, 9],
        [2, 5, 9],
        [3, 5, 6],
        [4, 6, 7]
    ];

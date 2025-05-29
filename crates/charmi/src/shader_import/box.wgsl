#define_import_path charmi::box

const LINES: array<u32, 16> = array(
    0x0u,    // Null
    0x2575u, // North
    0x2576u, // East
    0x2514u, // North+East
    0x2577u, // South
    0x2502u, // North+South
    0x250Cu, // East+South
    0x251Cu, // North+East+South
    0x2574u, // West
    0x2518u, // North+West
    0x2500u, // East+West
    0x2534u, // North+East+West
    0x2510u, // South+West
    0x2524u, // North+South+West
    0x252Cu, // East+South+West
    0x253Cu  // North+East+South+West
);
const NORTH = 1u;
const EAST = 2u;
const SOUTH = 4u;
const WEST = 8u;
const NORTHSOUTH = 5u;
const EASTWEST = 10u;


/*  ========================================================================  *
 *
 *      cubus.rs
 *      ~~~~~~~~
 *
 *      Simulation of Ernö Rubik's Cube
 *
 *      Target language:    Rust 0.11
 *
 *      Text encoding:      UTF-8
 *
 *      Created 2013-04-19: Ulrich Singer
 *
 *      $Id: cubus.rs 835 2014-07-19 09:53:35Z ucf $
 */

#![crate_name = "cubus"]

#![allow(unnecessary_parens)]
#![allow(unused_must_use)]

extern crate collections;
extern crate libc;

use libc::funcs::c95::stdlib::exit;

use collections::dlist::DList;
use collections::Deque;         // Trait for DList.
use collections::vec::Vec;

use std::io::{File, Open, Write};
use std::{io, os};
//use std::owned::Box;
//use std::boxed::BoxAny;         // Trait for Box.


/*  ––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––  *
 *
 *      Local Type System
 */


/// A short unsigned integer type for cube-local coordinate values.
/// Since the maximum cube size is 10, 4 bits would actually suffice.
type Coord = u8;


/// A brick location in a cube-local coordinate system.
#[deriving(PartialEq, Eq, Clone)]
struct Pos
{
    x: Coord,
    y: Coord,
    z: Coord

}   /* Pos */


/// Symbolic names for cube face colors.
#[deriving(Eq, Clone)]
enum Huename
{
    RD = 0x01,
    OR = 0x02,
    WT = 0x03,
    YL = 0x04,
    GN = 0x05,
    BL = 0x06

}   /* Huename */

impl PartialEq for Huename
{
    fn eq (&self, other: &Huename)
    -> bool
    {
        *self as u8 == *other as u8
    }

    fn ne (&self, other: &Huename)
    -> bool
    {
        *self as u8 != *other as u8
    }

}   /* PartialEq for Huename */


/// Face color distributions for a cube or a brick.
#[deriving(Eq, Clone)]
struct Hue
{
    xp: Huename,
    xn: Huename,
    yp: Huename,
    yn: Huename,
    zp: Huename,
    zn: Huename

}   /* Hue */

impl PartialEq for Hue
{
    fn eq (&self, other: &Hue)
    -> bool
    {
        self.xp == other.xp
     && self.yp == other.yp
     && self.zp == other.zp
    }

    fn ne (&self, other: &Hue)
    -> bool
    {
        self.xp != other.xp
     || self.yp != other.yp
     || self.zp != other.zp
    }

}   /* PartialEq for Hue */


/// Smallest movable cube fragment.
#[deriving(PartialEq, Eq, Clone)]
struct Brick
{
//  orgPos: Pos,
    curPos: Pos,
    curHue: Hue

}   /* Brick */

impl Brick
{
    /// Brick constructor.
    fn new (x: Coord, y: Coord, z: Coord)
    -> Brick
    {
        Brick {
//          orgPos: Pos { x: x, y: y, z: z },
            curPos: Pos { x: x, y: y, z: z },
            curHue: Hue { xp: RD, xn: OR, yp: WT, yn: YL, zp: GN, zn: BL }
        }

    }   /* ::new() */

}   /* impl Brick */


/// Rotates a brick counter-clockwise by 90° about the cube's X axis.
fn brick_rotated_x_pos (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcPos = &brick.curPos;
    let srcHue = &brick.curHue;
    Brick {
//      orgPos: brick.orgPos,
        curPos: Pos {
            x: srcPos.x,
            y: axmax - srcPos.z,
            z: srcPos.y
        },
        curHue: Hue {
            xp: srcHue.xp,
            xn: srcHue.xn,
            yp: srcHue.zn,
            yn: srcHue.zp,
            zp: srcHue.yp,
            zn: srcHue.yn
        }
    }

}   /* brick_rotated_x_pos() */


/// Rotates a brick clockwise by 90° about the cube's X axis.
fn brick_rotated_x_neg (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcPos = &brick.curPos;
    let srcHue = &brick.curHue;
    Brick {
//      orgPos: brick.orgPos,
        curPos: Pos {
            x: srcPos.x,
            y: srcPos.z,
            z: axmax - srcPos.y
        },
        curHue: Hue {
            xp: srcHue.xp,
            xn: srcHue.xn,
            yp: srcHue.zp,
            yn: srcHue.zn,
            zp: srcHue.yn,
            zn: srcHue.yp
        }
    }

}   /* brick_rotated_x_neg() */


/// Rotates a brick counter-clockwise by 90° about the cube's Y axis.
fn brick_rotated_y_pos (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcPos = &brick.curPos;
    let srcHue = &brick.curHue;
    Brick {
//      orgPos: brick.orgPos,
        curPos: Pos {
            x: srcPos.z,
            y: srcPos.y,
            z: axmax - srcPos.x
        },
        curHue: Hue {
            xp: srcHue.zp,
            xn: srcHue.zn,
            yp: srcHue.yp,
            yn: srcHue.yn,
            zp: srcHue.xn,
            zn: srcHue.xp
        }
    }

}   /* brick_rotated_y_pos() */


/// Rotates a brick clockwise by 90° about the cube's Y axis.
fn brick_rotated_y_neg (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcPos = &brick.curPos;
    let srcHue = &brick.curHue;
    Brick {
//      orgPos: brick.orgPos,
        curPos: Pos {
            x: axmax - srcPos.z,
            y: srcPos.y,
            z: srcPos.x
        },
        curHue: Hue {
            xp: srcHue.zn,
            xn: srcHue.zp,
            yp: srcHue.yp,
            yn: srcHue.yn,
            zp: srcHue.xp,
            zn: srcHue.xn
        }
    }

}   /* brick_rotated_y_neg() */


/// Rotates a brick counter-clockwise by 90° about the cube's Z axis.
fn brick_rotated_z_pos (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcPos = &brick.curPos;
    let srcHue = &brick.curHue;
    Brick {
//      orgPos: brick.orgPos,
        curPos: Pos {
            x: axmax - srcPos.y,
            y: srcPos.x,
            z: srcPos.z
        },
        curHue: Hue {
            xp: srcHue.yn,
            xn: srcHue.yp,
            yp: srcHue.xp,
            yn: srcHue.xn,
            zp: srcHue.zp,
            zn: srcHue.zn
        }
    }

}   /* brick_rotated_z_pos() */


/// Rotates a brick clockwise by 90° about the cube's Z axis.
fn brick_rotated_z_neg (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcPos = &brick.curPos;
    let srcHue = &brick.curHue;
    Brick {
//      orgPos: brick.orgPos,
        curPos: Pos {
            x: srcPos.y,
            y: axmax - srcPos.x,
            z: srcPos.z
        },
        curHue: Hue {
            xp: srcHue.yp,
            xn: srcHue.yn,
            yp: srcHue.xn,
            yn: srcHue.xp,
            zp: srcHue.zp,
            zn: srcHue.zn
        }
    }

}   /* brick_rotated_z_neg() */


/// A character type that designates a coordinate axis and a rotation direction.
type Axis = char;


/// A lambda type that returns a fixed coordinate component of a Pos.
type PosSelector<'a> = |&Pos|:'a -> Coord;


/// A function type that rotates a brick ±90° at a time around a fixed cube axis.
type BrickRotator = fn (&Brick, Coord) -> Brick;


/// Performs the indicated move on the given Brick vector
/// and returns a new vector in the resulting state.
fn brickvec_move (bricks: &[Brick], axdir: Axis, axval: Coord, axmax: Coord)
-> Vec<Brick>
{
    let selFun: PosSelector = match axdir
    {
        'X' | 'x'   =>  |pos: &Pos| pos.x,
        'Y' | 'y'   =>  |pos: &Pos| pos.y,
        'Z' | 'z'   =>  |pos: &Pos| pos.z,
        _           =>  fail!("Invalid axis designator {}", axdir)
    };

    let rotFun: BrickRotator = match axdir
    {
        'X' =>  brick_rotated_x_pos,
        'x' =>  brick_rotated_x_neg,
        'Y' =>  brick_rotated_y_pos,
        'y' =>  brick_rotated_y_neg,
        'Z' =>  brick_rotated_z_pos,
        'z' =>  brick_rotated_z_neg,
        _   =>  fail!("Invalid axis designator {}", axdir)
    };

    let mut newBricks: Vec<Brick> = vec![];
    for brick in bricks.iter()
    {
        if selFun(&brick.curPos) == axval
        {
            // Bricks in the affected layer are rotated.
            newBricks.push(rotFun(brick, axmax));
        }
        else
        {
            // Unaffected bricks are just copied.
            newBricks.push(*brick);
        }
    }

    newBricks

}   /* brickvec_move() */


/// Casts a move's identity as an integer, for fast equality tests.
fn make_move_ident (axdir: Axis, axval: Coord)
-> uint
{
    ((axdir as uint) << 4) | (axval as uint)

}   /* make_move_ident() */


/// A move on a cube, which is the rotation of a layer of bricks
/// around the selected cube axis by 90° at a time.  Affected bricks
/// are identified by their coordinate value on the rotation axis.
#[deriving(Eq, PartialEq, Clone)]
struct Move
{
    axdir:  Axis,
    axval:  Coord,
    ident:  uint

}   /* Move */


/// Returns a vector of Moves that were parsed from the given string.
fn string_to_movevec (string: &str, axmax: Coord)
-> Vec<Move>
{
    let maxChr = ('0' as u8 + axmax) as char;

    let mut moves: Vec<Move> = vec![];

    let mut count: uint = 1;
    let mut axdir: Axis = '_';
    let mut expectsAxis = true;
    let mut isInComment = false;
    for chr in string.as_slice().chars()
    {
        if isInComment
        {
            // Ignore until end.
            if chr == '\n'
            {
                isInComment = false;
            }
        }
        else
        if expectsAxis
        {
            if chr == 'X' || chr == 'x'
            || chr == 'Y' || chr == 'y'
            || chr == 'Z' || chr == 'z'
            {
                // Consume move axis.
                axdir = chr;

                expectsAxis = false;
            }
            else
            if '2' <= chr && chr <= '9'
            {
                // A prefixed digit acts as a repeat count.
                count = chr as uint - '0' as uint;
            }
            else
            if chr == '#'
            {
                isInComment = true;
            }
        }
        else
        {
            // Expecting a coordinate digit.
            if '0' <= chr && chr <= maxChr
            {
                let axval = (chr as u8 - '0' as u8) as Coord;

                let newMove = Move { axdir: axdir, axval: axval, ident: 0 };
                while count != 0
                {
                    moves.push(newMove);
                    count -= 1;
                }

                expectsAxis = true;
                count = 1;
            }
            else
            {
                fail!("Invalid coordinate value {}", chr);
            }
        }
    }

    moves

}   /* string_to_movevec() */


/// A Rubik's cube with a given edge length.
#[deriving(Eq, PartialEq, Clone)]
struct Cube
{
    size:   Coord,
    bricks: Vec<Brick>

}   /* Cube */

impl Cube
{
    /// Cube constructor.
    fn new (size: Coord)
    -> Cube
    {
        assert!(0 < size && size < 11);

        let axmax = size - 1;
        let mut bricks: Vec<Brick> = vec![];

        for z in range(0, size)
        {
            for y in range(0, size)
            {
                for x in range(0, size)
                {
                    // We're only interested in bricks that partake in the cube's surface.
                    if x == 0 || x == axmax
                    || y == 0 || y == axmax
                    || z == 0 || z == axmax
                    {
                        bricks.push(Brick::new(x, y, z));
                    }
                }
            }
        }

        Cube {
            size:   size,
            bricks: bricks
        }

    }   /* ::new() */

    /// Manipulates the receiving Cube instance according to the given Move
    /// sequence and returns a new Cube instance in the resulting state.
    fn move (&self, moves: &[Move])
    -> Cube
    {
        let size  = self.size;
        let axmax = size - 1;

        let mut bricks = self.bricks.clone();
        for move in moves.iter()
        {
            bricks = brickvec_move(bricks.as_slice(), move.axdir, move.axval, axmax);
        }

        Cube {
            size:   size,
            bricks: bricks
        }

    }   /* .move() */

}   /* impl Cube */


/*  ––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––  *
 *
 *      Move Finding
 */

/// A memory structure storing which layers have been moved.
struct Layers
{
    xpos: Vec<bool>,
    xneg: Vec<bool>,
    ypos: Vec<bool>,
    yneg: Vec<bool>,
    zpos: Vec<bool>,
    zneg: Vec<bool>

}   /* Layers */

impl Layers
{
    /// Layers constructor.
    fn new (size: uint)
    -> Layers
    {
        Layers {
            xpos: Vec::from_elem(size, false),
            xneg: Vec::from_elem(size, false),
            ypos: Vec::from_elem(size, false),
            yneg: Vec::from_elem(size, false),
            zpos: Vec::from_elem(size, false),
            zneg: Vec::from_elem(size, false)
        }

    }   /* ::new() */

    /// Sets the flag for the layer identified by axis and coordinate value.
    fn set_flag (&mut self, axdir: Axis, axval: Coord)
    {
        let axflags = match axdir
        {
            'X' =>  &mut self.xpos,
            'x' =>  &mut self.xneg,
            'Y' =>  &mut self.ypos,
            'y' =>  &mut self.yneg,
            'Z' =>  &mut self.zpos,
            'z' =>  &mut self.zneg,
            _   =>  fail!()
        };

        axflags.as_mut_slice()[axval as uint] = true;

    }   /* .set_flag() */

    /// Tests the flag for the layer identified by axis and coordinate value.
    fn has_flag (&self, axdir: Axis, axval: Coord)
    -> bool
    {
        let axflags = match axdir
        {
            'X' =>  &self.xpos,
            'x' =>  &self.xneg,
            'Y' =>  &self.ypos,
            'y' =>  &self.yneg,
            'Z' =>  &self.zpos,
            'z' =>  &self.zneg,
            _   =>  fail!()
        };

        axflags.as_slice()[axval as uint]

    }   /* .has_flag() */

}   /* impl Layers */


fn brickvec_eq (lhs: &[Brick], rhs: &[Brick])
-> bool
{
    let len = lhs.len();
    if rhs.len() != len
    {
        return false;
    }

    for ind in range(0, len)
    {
        if lhs[ind] != rhs[ind]
        {
            return false;
        }
    }

    true

}   /* brickvec_eq() */


/// A move sequence that transposed a set of bricks to their captured state.
#[deriving(Eq, PartialEq, Clone)]
struct Tracing
{
    bricks: Vec<Brick>,
    movstk: Vec<Move>

}   /* Tracing */


/// Returns the given axis with its rotational sense inverted.
fn invert_axis (axdir: Axis)
-> Axis
{
    // Flips uppercase <-> lowercase.
    ((axdir as u8) ^ 0x20) as Axis

}   /* invert_axis() */


/// Returns the string representation of the given move sequence.
fn movstk_to_string (movstk: &[Move])
-> String
{
    let mut string = String::with_capacity(2 * movstk.len());
    for move in movstk.iter().rev()
    {
        let chrseq = format!("{:c}{:u}", move.axdir, move.axval);
        string = string + chrseq;
    }

    string

}   /* movstk_to_string() */


/// Constructively inserts a new Move at the front of the given vector.
fn movstk_push (movstk: &[Move], axdir: Axis, axval: Coord, ident: uint)
-> Vec<Move>
{
    let mut newStack = movstk.into_owned();
    newStack.unshift(Move { axdir: axdir, axval: axval, ident: ident });

    newStack

}   /* movstk_push() */


/// Finds all move sequences, no longer than maxLen, that transform the
/// srcCube into the dstCube.
fn find_moves (maxLen: uint, srcCube: &Cube, dstCube: &Cube)
-> (Vec<String>, uint)
{
    let cubeSize = srcCube.size;
    if dstCube.size != cubeSize
    {
        fail!("Cubes are of different size");
    }

    let axmax = cubeSize - 1;

    let mut dblMovs = Layers::new(cubeSize as uint);
    let mut lastLen = 0;

    let mut traceQ: DList<Tracing> = DList::new();
    let oneTrc = Tracing {
        bricks: srcCube.bricks.clone(),
        movstk: vec![]
    };
    traceQ.push_back(oneTrc);

    let mut seqStrs: Vec<String> = vec![];
    let mut moveNum: uint = 0;

    // Process available tracings.
    while traceQ.len() != 0
    {
        let tracing = traceQ.pop_front().unwrap();

        // Does the tracing's move sequence produce the target state?
        if brickvec_eq(tracing.bricks.as_slice(), dstCube.bricks.as_slice())
        {
            // Collect successful target match and don't continue the tracing.
            let movStr = movstk_to_string(tracing.movstk.as_slice());
            seqStrs.push(movStr);
        }
        else
        {
            // Explore possible continuations of the tracing's move sequence.
            let movstk = &tracing.movstk;
            let trcLen = movstk.len();
            if trcLen < maxLen
            {
                let nxtLen = trcLen + 1;
                let bricks = &tracing.bricks;
                let mut negdir: Axis  = '_';
                let mut axval1: Coord = 0x0F;
                let mut ident1: uint  = 0x00;
                let mut ident2: uint  = 0x00;
                if trcLen > 0
                {
                    if trcLen > 1
                    {
                        ident2 = movstk.get(1).ident;
                    }

                    if trcLen > lastLen
                    {
                        dblMovs = Layers::new(cubeSize as uint);
                        lastLen = trcLen;
                    }

                    let move1 = &movstk.get(0);
                    negdir = invert_axis(move1.axdir);
                    axval1 = move1.axval;
                    ident1 = move1.ident;
                }

                // Systematically explore layer movements.
                for axdirRef in ['X', 'x', 'Y', 'y', 'Z', 'z'].iter()
                {
                    let axdir = *axdirRef;

                    for axval in range(0, cubeSize)
                    {
                        // Don't rotate a layer in the opposite direction of its previous move.
                        if trcLen > 0
                        && axval == axval1
                        && axdir == negdir
                        {
                            continue;
                        }

                        let ident = make_move_ident(axdir, axval);

                        // Don't rotate a layer in the same direction thrice.
                        if trcLen > 1
                        && ident == ident1
                        && ident == ident2
                        {
                            continue;
                        }

                        let isDbl = (trcLen > 0 && ident == ident1);

                        // Don't do a double move if the opposite double has been done.
                        if isDbl && dblMovs.has_flag(negdir, axval)
                        {
                            continue;
                        }

                        // Perform new exploratory move.
                        let nstate = brickvec_move(bricks.as_slice(), axdir, axval, axmax);

                        // Don't create a tracing for a final move in a sequence.
                        if nxtLen >= maxLen
                        {
                            // Compare a final move's outcome to the target state.
                            if brickvec_eq(nstate.as_slice(), dstCube.bricks.as_slice())
                            {
                                // Collect successful target match.
                                let nmoves = movstk_push(movstk.as_slice(), axdir, axval, ident);
                                let movStr = movstk_to_string(nmoves.as_slice());
                                seqStrs.push(movStr);
                            }
                        }
                        else
                        {
                            // Attempt to continue this move sequence.
                            let nmoves = movstk_push(movstk.as_slice(), axdir, axval, ident);
                            let newTrc = Tracing {
                                bricks: nstate,
                                movstk: nmoves
                            };
                            traceQ.push_back(newTrc);
                        }

                        if isDbl
                        {
                            // Register any double moves.
                            dblMovs.set_flag(axdir, axval);
                        }

                        // Count the exploratory moves actually performed.
                        moveNum += 1;
                    }
                }
            }
        }
    }

    (seqStrs, moveNum)

}   /* find_moves() */


/*  ––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––  *
 *
 *      Output Functions
 */


/// Returns a stream that writes output to the terminal.
fn tty_out ()
-> File
{
    match File::open_mode(&Path::new("/dev/tty"), Open, Write)
    {
        Ok(stream)  =>  stream,
        Err(error)  =>  fail!(error)
    }

}   /* tty_out() */


/// Saves the VT100 cursor position.
fn tty_save ()
{
    tty_out().write_str("\x1B7");

}   /* tty_save() */


/// Restores the VT100 cursor position.
fn tty_load ()
{
    tty_out().write_str("\x1B8");

}   /* tty_load() */


/// Writes output to the terminal at the given position.
fn tty_put_at (row: uint, col: uint, text: &str)
{
    tty_out().write_str(format!("\x1B[{:u};{:u}f{:s}", row, col, text).as_slice());

}   /* tty_put_at() */


/// Maps cube face color symbols to VT100 color control sequences.
fn huename_vt100_attrs (colorSym: Huename)
-> &'static str
{
    // "[2;30;40m" : Black
    // "[2;31;41m" : Red
    // "[2;32;42m" : Green
    // "[2;33;43m" : Yellow
    // "[2;34;44m" : Blue
    // "[2;35;45m" : Magenta
    // "[2;36;46m" : Cyan
    // "[1;37;47m" : White
    match colorSym
    {
        RD  => "\x1B[2;31;41m",
        OR  => "\x1B[2;36;46m",    // Using Cyan for Orange.
        WT  => "\x1B[1;37;47m",
        YL  => "\x1B[2;33;43m",
        GN  => "\x1B[2;32;42m",
        BL  => "\x1B[2;34;44m"
    }

}   /* huename_vt100_attrs() */


/// Draws a single cube brick to the terminal as a character graphic.
fn draw_brick (brick: &Brick, axmax: Coord, row: uint, col: uint)
{
    // The Unicode “FULL BLOCK” character as a string.
    static FULL1: &'static str = "█";
    static FULL2: &'static str = "██";
    static FULL3: &'static str = "███";
    static FULL9: &'static str = "█████████";

    fn put (tty: &mut File, row: uint, col: uint, attr: &str, text: &str)
    {
        tty.write_str(format!("\x1B7\x1B[{:u};{:u}f{:s}{:s}\x1B8",
                              row, col, attr, text).as_slice());
    }

    let brickPos = &brick.curPos;
    let brickHue = &brick.curHue;

    let posX = brickPos.x;
    let posY = brickPos.y;
    let posZ = brickPos.z;

    let bRow = -4 * posY as uint +  2 * posZ as uint + 4 * axmax as uint + row + 1;
    let bCol =  9 * posX as uint + -3 * posZ as uint + 3 * axmax as uint + col + 1;

    let tty = &mut tty_out();

    if posZ == axmax
    {
        let attr = huename_vt100_attrs(brickHue.zp);
        put(tty, bRow + 2, bCol +  0, attr, FULL9);
        put(tty, bRow + 3, bCol +  0, attr, FULL9);
        put(tty, bRow + 4, bCol +  0, attr, FULL9);
        put(tty, bRow + 5, bCol +  0, attr, FULL9);
    }

    if posY == axmax
    {
        let attr = huename_vt100_attrs(brickHue.yp);
        put(tty, bRow + 0, bCol +  2, attr, FULL9);
        put(tty, bRow + 1, bCol +  1, attr, FULL9);
    }

    if posX == axmax
    {
        let attr = huename_vt100_attrs(brickHue.xp);
        put(tty, bRow + 0, bCol + 11, attr, FULL1);
        put(tty, bRow + 1, bCol + 10, attr, FULL2);
        put(tty, bRow + 2, bCol +  9, attr, FULL3);
        put(tty, bRow + 3, bCol +  9, attr, FULL3);
        put(tty, bRow + 4, bCol +  9, attr, FULL2);
        put(tty, bRow + 5, bCol +  9, attr, FULL1);
    }

}   /* draw_brick() */


fn draw_cube (cube: &Cube, row: uint, col: uint)
{
    let size  = cube.size;
    let axmax = size - 1;
//  let boxW  = (3 + 4) * size as uint;
    let boxH  = (2 + 4) * size as uint;

    // «Clear Screen» «Reset Attributes»
    tty_put_at(boxH + row + 2, 0, "\x1B[2J\x1B[0m");

    tty_save();
    for brick in cube.bricks.iter()
    {
        if brick.curPos.x == axmax
        || brick.curPos.y == axmax
        || brick.curPos.z == axmax
        {
            draw_brick(brick, axmax, row, col);
        }
    }
    tty_load();

}   /* draw_cube() */


#[inline(never)]
unsafe
fn usage ()
{
    let msg =
"Usage:  cubus N Moves

Depicts a Rubik's cube of edge length ‘N’, after applying the given
Moves to an ordered state, as a character graphic in the terminal.

0 < N < 11.

‘Moves’ is a sequence of character pairs «axis»«coord» where «axis»
is one of X, Y, Z, x, y, z, denoting the rotation axis and direction.
Uppercase means +90° (counter-clockwise) and lowercase means -90°
(clockwise) rotation of a brick layer around the named «axis», where
the axis transfixes the center of the cube.  The rotated bricks are
addressed by «coord», which is a single decimal digit in the range
0 ≤ «coord» < N.  A move rotates all bricks whose coordinate value
along «axis» is «coord» in the direction that is indicated by the
uppercase/lowercase feature of «axis».  A «coord» value of 0 denotes
the leftmost / bottommost / hindmost cube layer.";

    io::stderr().write_str(msg);
    libc::funcs::c95::stdlib::exit(1);

    // NoReturn //

}   /* usage() */


/**
 *  Global entry point
 */
fn main ()
{
    let argv = os::args();
    let argc = argv.len();
    if argc < 2
    {
        unsafe { usage(); }
    }

    let mut size = match from_str::<i8>(argv.get(1).as_slice())
    {
        Some(value) => value,
        None        => 0
    };

    let mut doFindMoves = false;
    if size < 0
    {
        doFindMoves = true;
        size = -size;
    }
    let argCubeSize = size as u8;

    if argCubeSize < 1 || 10 < argCubeSize
    {
        unsafe { usage(); }
    }

    let argMoveStr = if argc > 2 {argv.slice_from(2).connect("\n")} else {"".to_string()};

    let argMoveVec = string_to_movevec(argMoveStr.as_slice(), argCubeSize - 1);

    let srcCube = Cube::new(argCubeSize);
    let dstCube = srcCube.move(argMoveVec.as_slice());
    draw_cube(&dstCube, 1, 2);

    println!("{:s}", argMoveStr);

    let maxLen = argMoveVec.len();
    if doFindMoves && maxLen != 0
    {
        let (foundVec, moveNum) = find_moves(maxLen, &srcCube, &dstCube);
        let foundNum = foundVec.len();
        println!("{:u} sequence{:s} from {:u} exploratory move{:s}:",
                 foundNum, if foundNum != 1 {"s"} else {""},
                 moveNum, if moveNum != 1 {"s"} else {""});

        let mut stepNum: uint = 0;
        for movStrRef in foundVec.iter()
        {
            if stepNum % 4 != 0
            {
                print!("\t");
            }

            print!("{}", *movStrRef);
            stepNum += 1;

            if stepNum % 4 == 0
            {
                print!("\n");
            }
        }
        if stepNum % 4 != 0
        {
            print!("\n");
        }
    }

}   /* main() */


/* ~ cubus.rs ~ */

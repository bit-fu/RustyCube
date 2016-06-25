/*  ========================================================================  *
 *
 *    cubus.rs
 *    ~~~~~~~~
 *
 *    Simulation of Ernö Rubik’s Cube
 *
 *    Target language:    Rust 1.6.0
 *
 *    Text encoding:      UTF-8
 *
 *    Created 2013-04-19: Ulrich Singer
 *
 *    $Id: cubus.rs 1320 2016-06-25 18:03:49Z ucf $
 */

#![crate_name = "cubus"]

#![allow(unused_parens)]
#![allow(unused_must_use)]

#![allow(non_snake_case)]


use std::collections::VecDeque;
use std::vec::Vec;
use std::env;

use std::{io, process};
use std::fs::{File, OpenOptions};
use std::io::Write;


/*  ––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––  */


/// A short unsigned integer type for cube-local coordinate values.
/// Since the maximum cube size is 10, 4 bits would actually suffice.
type Coord = u8;


/// A type that designates a coordinate axis and a rotation direction.
type Axis = char;


/// A brick location in a cube-local coordinate system.
#[derive(Eq, PartialEq, Copy, Clone)]
struct Loc
{
    x: Coord,
    y: Coord,
    z: Coord

}   /* Loc */

/// Component accessors.
fn get_x (loc: &Loc) -> Coord { loc.x }
fn get_y (loc: &Loc) -> Coord { loc.y }
fn get_z (loc: &Loc) -> Coord { loc.z }


/// Symbolic names for cube face colors.
#[derive(Eq, PartialEq, Copy, Clone)]
enum Huename
{
    RD = 0x01,
    OR = 0x02,
    WT = 0x03,
    YL = 0x04,
    GN = 0x05,
    BL = 0x06

}   /* Huename */

impl Huename
{
    /// Maps cube face color symbols to VT100 color control sequences.
    fn vt100_attrs (&self)
    -> &'static str
    {
        // "\e[2;30;40m" : Black
        // "\e[2;31;41m" : Red
        // "\e[2;32;42m" : Green
        // "\e[2;33;43m" : Yellow
        // "\e[2;34;44m" : Blue
        // "\e[2;35;45m" : Magenta
        // "\e[2;36;46m" : Cyan
        // "\e[1;37;47m" : White
        match *self
        {
            Huename::RD  => "\x1B[2;31;41m",
            Huename::OR  => "\x1B[2;36;46m",    // Using Cyan for Orange.
            Huename::WT  => "\x1B[1;37;47m",
            Huename::YL  => "\x1B[2;33;43m",
            Huename::GN  => "\x1B[2;32;42m",
            Huename::BL  => "\x1B[2;34;44m"
        }

    } /* .vt100_attrs() */

}   /* impl Huename */


/// Face color distributions for a cube or a brick.
#[derive(Eq, Copy, Clone)]
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

}   /* impl PartialEq for Hue */


/// Smallest movable cube fragment.
#[derive(Eq, PartialEq, Copy, Clone)]
struct Brick
{
    curLoc: Loc,
    curHue: Hue

}   /* Brick */

impl Brick
{
    /// Brick constructor.
    fn new (x: Coord, y: Coord, z: Coord)
    -> Brick
    {
        Brick {
            curLoc: Loc { x: x, y: y, z: z },
            curHue: Hue {
                xp: Huename::RD, xn: Huename::OR,
                yp: Huename::WT, yn: Huename::YL,
                zp: Huename::GN, zn: Huename::BL }
        }

    } /* ::new() */

}   /* impl Brick */


/// Rotates a brick counter-clockwise by 90° about the cube's X axis.
fn brick_rotated_x_pos (brick: &Brick, axmax: Coord)
-> Brick
{
    let srcLoc = &brick.curLoc;
    let srcHue = &brick.curHue;
    Brick {
        curLoc: Loc {
            x: srcLoc.x,
            y: axmax - srcLoc.z,
            z: srcLoc.y
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
    let srcLoc = &brick.curLoc;
    let srcHue = &brick.curHue;
    Brick {
        curLoc: Loc {
            x: srcLoc.x,
            y: srcLoc.z,
            z: axmax - srcLoc.y
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
    let srcLoc = &brick.curLoc;
    let srcHue = &brick.curHue;
    Brick {
        curLoc: Loc {
            x: srcLoc.z,
            y: srcLoc.y,
            z: axmax - srcLoc.x
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
    let srcLoc = &brick.curLoc;
    let srcHue = &brick.curHue;
    Brick {
        curLoc: Loc {
            x: axmax - srcLoc.z,
            y: srcLoc.y,
            z: srcLoc.x
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
    let srcLoc = &brick.curLoc;
    let srcHue = &brick.curHue;
    Brick {
        curLoc: Loc {
            x: axmax - srcLoc.y,
            y: srcLoc.x,
            z: srcLoc.z
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
    let srcLoc = &brick.curLoc;
    let srcHue = &brick.curHue;
    Brick {
        curLoc: Loc {
            x: srcLoc.y,
            y: axmax - srcLoc.x,
            z: srcLoc.z
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


/// Performs the indicated move on the given Brick vector
/// and returns a new vector in the resulting state.
fn brickvec_move (bricks: &[Brick], axdir: Axis, axval: Coord, axmax: Coord)
-> Vec<Brick>
{
    // A function that returns a fixed coordinate component of a Loc.
    let selFun: fn (&Loc) -> Coord =
    match axdir
    {
        'X' | 'x' =>  get_x,
        'Y' | 'y' =>  get_y,
        'Z' | 'z' =>  get_z,
        _         =>  panic!("Invalid axis designator {}", axdir)
    };

    // A function that rotates a brick ±90° at a time around a fixed cube axis.
    let rotFun: fn (&Brick, Coord) -> Brick =
    match axdir
    {
        'X' =>  brick_rotated_x_pos,
        'x' =>  brick_rotated_x_neg,
        'Y' =>  brick_rotated_y_pos,
        'y' =>  brick_rotated_y_neg,
        'Z' =>  brick_rotated_z_pos,
        'z' =>  brick_rotated_z_neg,
        _   =>  panic!("Invalid axis designator {}", axdir)
    };

    let mut newBricks: Vec<Brick> = Vec::with_capacity(bricks.len());
    for brick in bricks.iter()
    {
        if selFun(&brick.curLoc) == axval
        {
            // Bricks in the affected layer are rotated.
            newBricks.push(rotFun(brick, axmax));
        }
        else
        {
            // Unaffected bricks are just copied.
            newBricks.push(brick.clone());
        }
    }

    newBricks

}   /* brickvec_move() */


/// Casts a move's identity as an integer, for fast equality tests.
fn ident_of_move (axdir: Axis, axval: Coord)
-> u16
{
    (((axdir as u16) & 0x00FF) << 8) | (axval as u16)

}   /* ident_of_move() */


/// A move on a cube, which is the rotation of a layer of bricks
/// around the selected cube axis by 90° at a time.  Affected bricks
/// are identified by their coordinate value on the rotation axis.
#[derive(Eq, PartialEq, Copy, Clone)]
struct Move
{
    axdir:  Axis,
    axval:  Coord,
    ident:  u16

}   /* Move */


/// Returns a vector of Moves that were parsed from the given string.
fn movevec_of_string (string: &str, axmax: Coord)
-> Vec<Move>
{
    let maxChr = ('0' as u8 + axmax) as char;

    let mut moves: Vec<Move> = vec![];

    let mut count: u8 = 1;
    let mut axdir: Axis = '_';
    let mut expectsAxis = true;
    let mut isInComment = false;
    for chr in string.chars()
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
                count = (chr as u8 - '0' as u8) % 4u8;
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
                    moves.push(newMove.clone());
                    count -= 1;
                }

                expectsAxis = true;
                count = 1;
            }
            else
            {
                panic!("Invalid coordinate value {}", chr);
            }
        }
    }

    moves

}   /* movevec_of_string() */


/// A Rubik's cube with a given edge length.
#[derive(Eq, PartialEq, Clone)]
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

        for z in 0 .. size
        {
            for y in 0 .. size
            {
                for x in 0 .. size
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

    } /* ::new() */

    /// Manipulates the receiving Cube instance according to the given Move
    /// sequence and returns a new Cube instance in the resulting state.
    fn copy_with_moves (&self, moves: &[Move])
    -> Cube
    {
        let size  = self.size;
        let axmax = size - 1;

        let mut bricks = self.bricks.clone();
        for mov in moves.iter()
        {
            bricks = brickvec_move(&bricks, mov.axdir, mov.axval, axmax);
        }

        Cube {
            size:   size,
            bricks: bricks
        }

    } /* .copy_with_moves() */

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

// Vec allocation helper.
fn vec_of_size<T: Clone> (size: usize, value: T)
-> Vec<T>
{
    let mut vec: Vec<T> = vec![];
    vec.resize(size, value);

    vec

}   /* vec_of_size<T>() */

impl Layers
{
    /// Layers constructor.
    fn new (size: Coord)
    -> Layers
    {
        let size: usize = size as usize;
        Layers {
            xpos: vec_of_size(size, false),
            xneg: vec_of_size(size, false),
            ypos: vec_of_size(size, false),
            yneg: vec_of_size(size, false),
            zpos: vec_of_size(size, false),
            zneg: vec_of_size(size, false)
        }

    } /* ::new() */

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
            _   =>  panic!()
        };

        axflags[axval as usize] = true;

    } /* .set_flag() */

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
            _   =>  panic!()
        };

        axflags[axval as usize]

    } /* .has_flag() */

}   /* impl Layers */


fn brickvec_eq (lhs: &[Brick], rhs: &[Brick])
-> bool
{
    let len = lhs.len();
    if rhs.len() != len
    {
        return false;
    }

    for ind in 0 .. len
    {
        if lhs[ind] != rhs[ind]
        {
            return false;
        }
    }

    true

}   /* brickvec_eq() */


/// An experimental move sequence in reverse, so the most recent moves are easily accessible.
struct Trail
{
    steps: Vec<Move>

}   /* Trail */

impl Trail
{
    /// Trail constructor.
    fn new ()
    -> Trail
    {
        Trail { steps: vec![] }

    }   /* ::new() */

    /// Returns a sequel with the given next move in front.
    fn proceed (&self, axdir: Axis, axval: Coord, ident: u16)
    -> Trail
    {
        let mut sequel: Vec<Move> = Vec::with_capacity(self.steps.len() + 1);

        sequel.push(Move { axdir: axdir, axval: axval, ident: ident });
        sequel.extend(self.steps.iter().cloned());

        Trail { steps: sequel }

    }   /* .proceed() */

    /// Returns the cube brick configuration produced by this Trail.
    fn transform (&self, bricks: &Vec<Brick>, axmax: Coord)
    -> Vec<Brick>
    {
        let mut bricks = bricks.clone();
        for mov in self.steps.iter().rev()
        {
            bricks = brickvec_move(&bricks, mov.axdir, mov.axval, axmax);
        }

        bricks

    }   /* .transform() */

    /// Returns the Trail's string representation.
    fn as_string (&self)
    -> String
    {
        let mut string = String::with_capacity(2 * self.steps.len());
        for mov in self.steps.iter().rev()
        {
            string = string + &format!("{}{}", mov.axdir, mov.axval);
        }

        string

    }   /* .as_string() */

}   /* impl Trail */


/// Returns the given axis with its rotational sense inverted.
fn invert_axis (axdir: Axis)
-> Axis
{
    // Flips uppercase <-> lowercase.
    ((axdir as u8) ^ 0x20) as Axis

}   /* invert_axis() */


/// Finds all move sequences, no longer than maxLen, that transform the
/// srcCube into the dstCube.
fn find_moves (maxLen: usize, srcCube: &Cube, dstCube: &Cube)
-> (Vec<String>, u64)
{
    let cubeSize = srcCube.size;
    if dstCube.size != cubeSize
    {
        panic!("Cubes are of different size");
    }

    let axmax = cubeSize - 1;

    let mut dblMovs = Layers::new(cubeSize);
    let mut lastLen = 0;

    let mut trailQ: VecDeque<Trail> = VecDeque::new();
    trailQ.push_back(Trail::new());

    let mut seqStrs: Vec<String> = vec![];
    let mut moveNum: u64 = 0;

    // Process available trails.
    while trailQ.len() != 0
    {
        let trail = trailQ.pop_front().unwrap();
        let bricks = trail.transform(&srcCube.bricks, axmax);

        // Does the trail's move sequence produce the target state?
        if brickvec_eq(&bricks, &dstCube.bricks)
        {
            // Collect successful target match and don't continue the trail.
            seqStrs.push(trail.as_string());
        }
        else
        {
            // Explore possible continuations of the trail's move sequence.
            let movStack: &[Move] = &trail.steps;
            let trailLen = movStack.len();
            if trailLen < maxLen
            {
                let mut negdir: Axis  = '_';
                let mut axval1: Coord = 0x0F;
                let mut ident1: u16   = 0x00;
                let mut ident2: u16   = 0x00;
                if trailLen > 0
                {
                    if trailLen > 1
                    {
                        ident2 = movStack[1].ident;
                    }

                    if trailLen > lastLen
                    {
                        dblMovs = Layers::new(cubeSize);
                        lastLen = trailLen;
                    }

                    let move1 = &movStack[0];
                    negdir = invert_axis(move1.axdir);
                    axval1 = move1.axval;
                    ident1 = move1.ident;
                }

                // Systematically explore layer movements.
                for axdirRef in ['X', 'x', 'Y', 'y', 'Z', 'z'].iter()
                {
                    let axdir = *axdirRef;

                    for axval in 0 .. cubeSize
                    {
                        // Don't rotate a layer in the opposite direction of its previous move.
                        if trailLen > 0
                        && axval == axval1
                        && axdir == negdir
                        {
                            continue;
                        }

                        let ident = ident_of_move(axdir, axval);

                        // Don't rotate a layer in the same direction thrice.
                        if trailLen > 1
                        && ident == ident1
                        && ident == ident2
                        {
                            continue;
                        }

                        // Is the candidate move a duplicate of the most recent move in this trail?
                        let isDbl = (trailLen > 0 && ident == ident1);

                        // Don't do a double move if the opposite double has been done.
                        if isDbl && dblMovs.has_flag(negdir, axval)
                        {
                            continue;
                        }

                        if trailLen >= axmax as usize
                        {
                            // Check if all layers rotate identically.  This would be equivalent
                            // to a rotation of the cube as a whole.  Such a transformation is too
                            // trivial to be used as a basis for meaningful alternative moves.
                            let mut sameDir: bool = true;
                            for ind in 0 .. axmax as usize
                            {
                                if movStack[ind].axdir != axdir
                                {
                                    sameDir = false;
                                    break
                                }
                            }
                            if sameDir
                            {
                                let mut usedVal: Vec<bool> = vec_of_size(cubeSize as usize, false);
                                usedVal[axval as usize] = true;
                                for ind in 0 .. axmax as usize
                                {
                                    usedVal[movStack[ind].axval as usize] = true
                                }

                                let mut usedAll = true;
                                for ind in 0 .. cubeSize as usize
                                {
                                    if ! usedVal[ind]
                                    {
                                        usedAll = false;
                                        break
                                    }
                                }
                                if usedAll
                                {
                                    // Skip cube rotation.
                                    continue
                                }
                            }
                        }

                        // Perform new exploratory move.
                        let ntrail = trail.proceed(axdir, axval, ident);

                        // Attempt to continue this move sequence.
                        trailQ.push_back(ntrail);

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
    match OpenOptions::new().create(true).write(true).open("/dev/tty")
    {
        Ok(stream)  =>  stream,
        Err(error)  =>  panic!(error)
    }

}   /* tty_out() */


/// Saves the VT100 cursor position.
fn tty_save ()
{
    write!(tty_out(), "\x1B7");

}   /* tty_save() */


/// Restores the VT100 cursor position.
fn tty_load ()
{
    write!(tty_out(), "\x1B8");

}   /* tty_load() */


/// Writes output to the terminal at the given position.
fn tty_put_at (row: i16, col: i16, text: &str)
{
    write!(tty_out(), "\x1B[{};{}f{}", row, col, text);

}   /* tty_put_at() */


/// Draws a single cube brick to the terminal as a character graphic.
fn draw_brick (brick: &Brick, axmax: Coord, row: i16, col: i16)
{
    // The Unicode “FULL BLOCK” character as a string.
    static FULL1: &'static str = "█";
    static FULL2: &'static str = "██";
    static FULL3: &'static str = "███";
    static FULL9: &'static str = "█████████";

    fn put (tty: &mut File, row: i16, col: i16, attr: &str, text: &str)
    {
        write!(tty, "\x1B7\x1B[{};{}f{}{}\x1B8", row, col, attr, text);
    }

    let axmax = axmax  as i16;

    let brickLoc = &brick.curLoc;
    let brickHue = &brick.curHue;

    let posX = brickLoc.x as i16;
    let posY = brickLoc.y as i16;
    let posZ = brickLoc.z as i16;

    let bRow = -4 * posY +  2 * posZ + 4 * axmax + row + 1;
    let bCol =  9 * posX + -3 * posZ + 3 * axmax + col + 1;

    let tty = &mut tty_out();

    if posZ == axmax
    {
        let attr = brickHue.zp.vt100_attrs();
        put(tty, bRow + 2, bCol +  0, attr, FULL9);
        put(tty, bRow + 3, bCol +  0, attr, FULL9);
        put(tty, bRow + 4, bCol +  0, attr, FULL9);
        put(tty, bRow + 5, bCol +  0, attr, FULL9);
    }

    if posY == axmax
    {
        let attr = brickHue.yp.vt100_attrs();
        put(tty, bRow + 0, bCol +  2, attr, FULL9);
        put(tty, bRow + 1, bCol +  1, attr, FULL9);
    }

    if posX == axmax
    {
        let attr = brickHue.xp.vt100_attrs();
        put(tty, bRow + 0, bCol + 11, attr, FULL1);
        put(tty, bRow + 1, bCol + 10, attr, FULL2);
        put(tty, bRow + 2, bCol +  9, attr, FULL3);
        put(tty, bRow + 3, bCol +  9, attr, FULL3);
        put(tty, bRow + 4, bCol +  9, attr, FULL2);
        put(tty, bRow + 5, bCol +  9, attr, FULL1);
    }

}   /* draw_brick() */


fn draw_cube (cube: &Cube, row: i16, col: i16)
{
    let size    = cube.size;
    let axmax = size - 1;
//  let boxW    = (3 + 4) * size as i16;
    let boxH    = (2 + 4) * size as i16;

    // «Clear Screen» «Reset Attributes»
    tty_put_at(boxH + row + 2, 0, "\x1B[2J\x1B[0m");

    tty_save();
    for brick in cube.bricks.iter()
    {
        if brick.curLoc.x == axmax
        || brick.curLoc.y == axmax
        || brick.curLoc.z == axmax
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
the leftmost / bottommost / hindmost cube layer.\n";

    write!(io::stderr(), "{}", msg);
    process::exit(1);

    // NoReturn //

}   /* usage() */


/**
 *  Global entry point
 */
fn main ()
{
    let argc = env::args().count();
    if argc < 2
    {
        unsafe { usage(); }
    }

    let mut size = match env::args().nth(1).unwrap().parse::<i8>()
    {
        Ok(value) => value,
        Err(_)    => 0
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

    let argMoveStr = if argc > 2 {env::args().skip(2).collect::<Vec<String>>().join("\n")} else {"".to_string()};

    let argMoveVec = movevec_of_string(&argMoveStr, argCubeSize - 1);

    let srcCube = Cube::new(argCubeSize);
    let dstCube = srcCube.copy_with_moves(&argMoveVec);
    draw_cube(&dstCube, 1, 2);

    println!("{}", argMoveStr);

    let maxLen = argMoveVec.len();
    if doFindMoves && maxLen != 0
    {
        let (foundVec, moveNum) = find_moves(maxLen, &srcCube, &dstCube);
        let foundNum = foundVec.len();
        println!("{} sequence{} from {} exploratory move{}:",
                 foundNum, if foundNum != 1 {"s"} else {""},
                 moveNum, if moveNum != 1 {"s"} else {""});

        let mut stepNum: u64 = 0;
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

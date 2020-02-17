#![allow(warnings)]
// #![windows_subsystem = "windows"]
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
extern crate rayon;
#[macro_use]
extern crate conrod;

mod engine;
mod game;
mod utils;

use crate::game::game::Game;

fn main() {
    let mut game = Game::new("Cave game v0.1.0");
    game.run();
}
// const CHUNKSIZE: usize = 16;
//
// fn main() {
//     let mut blocks: [[[u8; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE] = [[[0u8; 16]; 16]; 16];
//     // for z in 0..CHUNKSIZE{
//     //     for y in 0..CHUNKSIZE{
//     //         for x in 0..CHUNKSIZE{
//     //             if x == 0 && z == 0{
//     //                 blocks[x][y][z] = 1u8;
//     //             }
//     //         }
//     //     }
//     // }
//     blocks[0][0][0] = 1;
//     blocks[1][0][0] = 1;
//
//     let get_block = |x: isize, y: isize, z: isize| -> u8{
//         if x < 0 || x > 15{ return 0u8 }
//         if y < 0 || y > 15{ return 0u8 }
//         if z < 0 || z > 15{ return 0u8 }
//         blocks[x as usize][y as usize][z as usize]
//     };
//
//     for backface in &[false, true]{
//         for dim in 0..3{
//             let u = (dim + 1) % 3;
//             let v = (dim + 2) % 3;
//             let mut dir = [0, 0, 0];
//             dir[dim] = if !*backface {-1} else{1};
//
//             let mut current = [0isize, 0, 0];
//
//             // goes through each 'layer' of blocks in that dim
//             for layer in 0..CHUNKSIZE as isize{
//                 let mut mask = [[false; CHUNKSIZE]; CHUNKSIZE];
//                 current[dim] = layer; //sets the current layer
//                 for d1 in 0..CHUNKSIZE as isize{
//                     current[v] = d1;
//                     for d2 in 0..CHUNKSIZE as isize{
//                         current[u] = d2;
//                         let current_block = get_block(current[0], current[1], current[2]);
//
//                         let (mut w, mut h) = (1, 1);
//                         // if not masked already, not air and facing air
//                         if !mask[d1 as usize][d2 as usize] && current_block != 0u8 && get_block(current[0]+dir[0], current[1]+dir[1], current[2]+dir[2]) == 0u8{
//                             // println!("Current: {:?}:{:?}", current, current_block);
//                             mask[d1 as usize][d2 as usize] = true;
//                             let mut next = current;
//                             next[u] += 1;
//                             // if next block is equal current block, start increasing mesh size and not meshed already too...
//                             if current_block == get_block(next[0], next[1], next[2]) && !mask[d1 as usize][(d2+1) as usize]{
//                                 w += 1;
//                                 mask[d1 as usize][(d2+1) as usize] = true;
//                                 for i in d2+2..CHUNKSIZE as isize{ // for each remaining block in the current row
//                                     let mut next2 = next;
//                                     next2[u] = i;
//                                     if get_block(next2[0], next2[1], next2[2]) == current_block{ w += 1; mask[d1 as usize][i as usize] = true; /*println!("mask: {:?}", mask)*/} else { break }
//                                 }
//                             }
//
//                             'row: for j in d1+1..CHUNKSIZE as isize{ // for each row in the remaining rows
//                                 let mut next2 = next;
//                                 next2[v] = j;
//                                 for i in d2..d2+w as isize{ // for each remaining block in the current row
//                                     next2[u] = i;
//                                     if get_block(next2[0], next2[1], next2[2]) != current_block { break 'row }
//                                 }
//                                 for i in d2..CHUNKSIZE as isize{
//                                     mask[j as usize][i as usize] = true;
//                                 }
//                                 // println!("mask: {:?}", mask);
//                                 h += 1;
//                             }
//                             //mesh
//                             if dim==0{
//                                 println!("MeshPos: {:?} Size: {:?}", current, (h, w));
//                             }else{
//                                 println!("MeshPos: {:?} Size: {:?}", current, (w, h));
//                             }
//                         }
//
//                     }
//                 }
//
//             }
//             // println!("Dim: {:?} U: {:?} V: {:?}", dim, v, u);
//         }
//     }
// }

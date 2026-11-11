//! Fixed Rush Hour-style emergency route. A slide of any distance is one move.
use std::collections::{HashMap, VecDeque};
#[derive(Clone, Copy, Debug)]
pub struct Vehicle { pub horizontal: bool, pub lane: u8, pub length: u8 }
pub const VEHICLES: [Vehicle;8]=[
    Vehicle{horizontal:true,lane:2,length:2},
    Vehicle{horizontal:true,lane:0,length:2},
    Vehicle{horizontal:false,lane:5,length:3},
    Vehicle{horizontal:false,lane:2,length:3},
    Vehicle{horizontal:true,lane:3,length:3},
    Vehicle{horizontal:false,lane:0,length:2},
    Vehicle{horizontal:false,lane:4,length:2},
    Vehicle{horizontal:true,lane:5,length:3},
];
pub const INITIAL:[u8;8]=[0,0,0,1,3,4,4,1];
#[derive(Clone, Debug)]
pub struct Traffic {pub positions:[u8;8],pub history:Vec<[u8;8]>,pub hint:bool}
impl Default for Traffic {fn default()->Self {Self{positions:INITIAL,history:vec![],hint:false}}}
impl Traffic {
    pub fn preview(&self,index:usize,delta:i32)->u8 { Self::destination(self.positions,index,delta) }
    fn destination(state:[u8;8],index:usize,delta:i32)->u8 {
        if index>=8 {return 0;}
        let car=VEHICLES[index]; let mut position=state[index] as i32;
        let direction=delta.signum();
        for _ in 0..delta.unsigned_abs().min(6) {
            let next=position+direction;
            if next<0 || next>if index==0 {6}else{6-car.length as i32} {break;}
            let mut blocked=false;
            for k in 0..car.length as i32 {
                let (x,y)=if car.horizontal {(next+k,car.lane as i32)}else{(car.lane as i32,next+k)};
                if x>=6 && index==0 {continue;}
                for (j,other) in VEHICLES.iter().enumerate() {
                    if j==index {continue;}
                    for n in 0..other.length as i32 {
                        let cell=if other.horizontal {(state[j] as i32+n,other.lane as i32)}else{(other.lane as i32,state[j] as i32+n)};
                        if (x,y)==cell {blocked=true;}
                    }
                }
            }
            if blocked {break;}position=next;
        }
        // Crossing the exit commits a complete escape, never strands half a bus.
        if index==0 && position>4 {6}else{position as u8}
    }
    pub fn slide(&mut self,index:usize,delta:i32)->bool {
        if index>=8 || self.positions[0]==6 {return false;}
        let next=self.preview(index,delta);
        if next==self.positions[index] {return false;}
        self.history.push(self.positions);self.positions[index]=next;true
    }
    pub fn undo(&mut self) {if let Some(state)=self.history.pop(){self.positions=state;}}
    /// Exhaustive shortest-path validation, never displayed automatically.
    pub fn solution(&self)->Option<Vec<(usize,i32)>> {
        let mut queue=VecDeque::from([self.positions]);
        let mut parent=HashMap::from([(self.positions,None)]);
        while let Some(state)=queue.pop_front() {
            if state[0]==6 {
                let mut path=vec![];let mut s=state;
                while let Some((previous,index,delta))=parent[&s] {path.push((index,delta));s=previous;}
                path.reverse();return Some(path);
            }
            for index in 0..8 {for delta in -6..=6 {
                let next=Self::destination(state,index,delta);
                if next==state[index] {continue;}
                let mut candidate=state;candidate[index]=next;
                if let std::collections::hash_map::Entry::Vacant(e)=parent.entry(candidate) {
                    e.insert(Some((state,index,delta)));queue.push_back(candidate);
                }
            }}
        }
        None
    }
}

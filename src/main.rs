#![allow(unused_parens)]

// By Icedude907
// Typed up using vim on Termux with an android touch keyboard without a language server bcos i hate myself

use indoc::indoc;
use std::io::Write;
use termcolor::{BufferWriter, Buffer as TermBuffer, 
    WriteColor, Color as TermColor, ColorChoice, ColorSpec};

#[derive(Copy, Clone, Debug)]
struct Point{x: f64, y: f64}
type PointDelta = Point;

struct Entry{
    location: Point,
    data: EntryData,
    isgap: bool,
}
#[derive(Debug)]
enum EntryData{
    Sphere{d: f64}, // diameter
    Rect{w: f64, h: f64},
    Tringle{pt2: PointDelta, pt3: PointDelta}
}

impl Entry{
    fn centroid(&self) -> Point{
        return match self.data{
            EntryData::Sphere{..} => {
               self.location 
            }
            EntryData::Rect{w, h} => {
                Point{
                    x: self.location.x + w / 2.0,
                    y: self.location.y + h / 2.0,
                }
            }
            EntryData::Tringle{pt2, pt3} => {
                Point{
                    x: self.location.x + (pt2.x + pt3.x) / 3.0,
                    y: self.location.y + (pt2.y + pt3.y) / 3.0,
                }
            }
        }
    }
    fn area(&self) -> f64{
        let flatarea = match self.data{
            EntryData::Sphere{d} => {
                (d/2.0).powi(2) * std::f64::consts::PI
            }
            EntryData::Rect{w, h} => {
                w * h
            }
            EntryData::Tringle{pt2, pt3} => {
                // Area = 1/2[x1(y2 - y3) + x2(y3 - y1) + x3(y1 - y2)]
                // 0.5 * (0*... + pt2.x*(pt3.y-0) + pt3.x(0-pt2.y))
                (pt2.x * pt3.y - pt3.x * pt2.y) / 2.0
            }
        };
        return flatarea.abs();
    }
    fn coloured_string(&self) -> TermBuffer{
        let sign = if(self.isgap){"-"}else{"+"};
        let mut cb = coloured_by_bool(self.isgap);
        write!(&mut cb, "{}, {:?}, {:?}", sign, self.location, self.data);
        cb.reset();
        cb
    }
}

// Borrowed ty rust-lang forums 
macro_rules! printfl {
    ( $($t:tt)* ) => {
        {
            use std::io::Write;
            let mut h = std::io::stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}

#[derive(Default)]
struct State{
    entries: Vec<Entry>,
}

impl State{
    
    fn printEntries(&self){
        if(self.entries.len() == 0){ println!("[] (no entries)"); }
        let printer = BufferWriter::stdout(ColorChoice::Auto);
        for (i, e) in self.entries.iter().enumerate(){
            print!("{}: ", i);
            printer.print(&e.coloured_string());
            println!(",");
        }
    }

    fn printTableSolver(&self){
        // |xcen|ycen|Area| x*A | y*A |
        // |--------------------------|
        // ....
        // sums
        // A: -
        // xA: -
        // yA:
        // centroid (x, y)
        let mut sum_A = 0.0;
        let mut sum_xA = 0.0;
        let mut sum_yA = 0.0;
        println!(indoc!("
            | Area     | x cen |   xc * A   | y cen |   yc * A   |
            |----------------------------------------------------|"
        )); 
        let printer = BufferWriter::stdout(ColorChoice::Auto);
        for (i, e) in self.entries.iter().enumerate(){
            // The slow math
            let centroid = e.centroid();
            let area = e.area();
            let xA = centroid.x * area;
            let yA = centroid.y * area;
            
            // Print row
            let mut rowbuf = coloured_by_bool(e.isgap);
            // Making use of padding/align (>8)
            // and truncate / f64 precision (.3)
            // TODO: Crop decimals if large
            let largecolmaxchars = 10;
            let areaintchars = area.max(1.0).log10().floor() as isize + 1;
            let areadecimals = largecolmaxchars-areaintchars-1;
            let areaprecision = 3.min(areadecimals).max(0) as usize;
            writeln!(&mut rowbuf, "|{:>10.*}|{:>7.3}|{:>12.3}|{:>7.3}|{:>12.3}|",
                     areaprecision, area, centroid.x, xA, centroid.y, yA);
            rowbuf.reset();
            printer.print(&rowbuf);

            // totals
            let gapmul = [1.0f64, -1.0f64][e.isgap as usize]; // lookup
            sum_A += area * gapmul;
            sum_xA += xA  * gapmul;
            sum_yA += yA  * gapmul;
        }
        let centroid = Point{
            x: sum_xA / sum_A,
            y: sum_yA / sum_A,
        };
        println!(indoc!("
            Sum Area: {}
            Sum individual x cen * A: {}
            Sum individual y cen * A: {}
              dividing by the total area
            Final centroid: {:?}"
            ), 
            sum_A, sum_xA, sum_yA, centroid
        );
    }
}

fn main() {
    let help = indoc!{"
        Yo centroid solver time!. Commands:
        help: this
        status: Prints the entries in memory
        solve: Solves the current state
        add <+/-> <type> ...
           +/-: Whether this is solid material or a cut out
           type: One of 'circle' 'quad' 'tri'
           For 'circle': (centre point) radius
           For 'quad': (corner point) width height {can be -ve}
           For 'tri': (corner point) (point 2 delta) (point 3 delta from corner)
           Where a point is written as '(x, y)'
        remove idx: Removes an entry from the list
        quit: stop program
        Keep in mind the cutouts don't actually do collision checks for double counting, etc.
        Anyway, gl. (Program by Icedude_907)
        ------------------------------------"
    };
    println!("{}", help);
    
    let mut state = State::default();

    loop{
        printfl!(">>> ");
        let inp = next_line();
        let inp = inp.trim();
        let (kwd, cmd) = inp.split_once(' ').unwrap_or((inp, ""));
        let mut kwd = kwd.to_owned(); kwd.make_ascii_lowercase();
        
        if(kwd == "quit"){
            std::process::exit(0);
        }else if(kwd == "help"){
            println!("{}", help);
        }else if(kwd == "status"){
            state.printEntries();
        }else if(kwd == "solve" || kwd.starts_with("eval")){
            state.printTableSolver();
        }else if(kwd == "rem" || kwd == "remove"){
            let idx = cmd.parse::<usize>();
            let idx = match idx{
                Err(_) => { println!("Err, not a number."); continue; },
                Ok(a) => a,
            };
            if(idx >= state.entries.len()){
                println!("Err, out of range");
                continue;
            }
            state.entries.remove(idx);
        }else if(kwd == "add"){
            let res = do_add_fn(cmd);
            match res{
                Err(s) => { println!("Err: {}", s); }
                Ok(e) => { 
                    state.entries.push(e);
                    println!("Added");
                }
            }

        }else{
            println!("Err, not a command");
        }
    }
    
}

fn next_line() -> String{
    let mut s = String::new();
    std::io::stdin().read_line(&mut s);
    s
}

struct Addfn<'a>{
    s: &'a str,
}
impl<'a> Addfn<'a>{
    // starts_with / strip_prefix then trim
    fn chew_start_if(&mut self, pat: &'_ str) -> bool
      /*where P: std::str::pattern::Pattern<'a>*/{
        let r = self.chew_start_if_notrim(pat);
        self.s = self.s.trim_start();
        return r;
    }
    fn chew_start_if_notrim(&mut self, pat: &'_ str) -> bool
      /*where P: std::str::pattern::Pattern<'a>*/{
        let m = self.s.strip_prefix(pat);
        if let Some(s) = m { self.s = s;}
        return m.is_some();
    }
    /// Returns Err if not a float
    fn chew_float(&mut self) -> Result<f64, ()> {
        let negative = self.chew_start_if_notrim("-") as usize;
        // Todo, no trim between signs
        let sign = [1.0, -1.0][negative];
        let (f, n) = fast_float::parse_partial::<f64, _>(self.s).map_err(|_| ())?;
        self.s = &self.s[n..].trim_start();
        return Ok(f*sign);
    }
}
impl<'a> std::ops::Deref for Addfn<'a>{
    type Target = str;
    fn deref(&self) -> &Self::Target{
        &self.s
    }
}
fn do_add_fn(cmd: &str) -> Result<Entry, &'static str>{
    // trim_front between each
    // this isnt a reusable parser, just a once off. Code quality poor.
    let mut work = Addfn{s: cmd.trim_start() };
    let isgap = {
        if work.chew_start_if("+") {false}
        else if work.chew_start_if("-") {true}
        else{ return Err("no sign");}
    };
    println!("{}", work.s);
    enum ParseType{Circle, Rectangle, Triangle}
    let typ = if(work.chew_start_if("circle")){
            ParseType::Circle
        }else if(work.chew_start_if("quad")){
            ParseType::Rectangle
        }else if(work.chew_start_if("tri")){
            ParseType::Triangle
        }else{
            return Err("what shape?");
        };

    //<trim>(<trim>x<,><trim>y<trim>)
    let mut read_point = || -> Result<Point, &'static str>{
        let open = work.chew_start_if("(");
        if(!open){
            return Err("expected point '('");
        } 
        let x = work.chew_float().map_err(|_| "X is not a number" )?; // TODo
        work.chew_start_if(",");
        let y = work.chew_float().map_err(|_| "Y is not a number" )?; // TODo
        let close = work.chew_start_if(")");
        if(!close){
            return Err("close point pls");
        }
        Ok(Point{x, y})
    };
    
    let location = read_point()?;

    let data: EntryData = match typ{
        ParseType::Circle => {
            // diameter
            let work = work.trim();
            let d = work.parse::<f64>().map_err(|_| "diameter is not a num")?;
            EntryData::Sphere{d}
        }
        ParseType::Rectangle => {
            // width
            let w = work.chew_float().map_err(|_| "width is not a num")?;
            let h = work.chew_float().map_err(|_| "height is not a num")?;
            EntryData::Rect{w, h}
        }
        ParseType::Triangle => {
            let pt2 = read_point()?;
            let pt3 = read_point()?;
            EntryData::Tringle{pt2, pt3}
        }
        // _ => { unimplemented!() }
    };

    return Ok(Entry{
        isgap, location, data
    });
}

fn coloured_by_bool(isRed: bool) -> TermBuffer{
    let mut t = TermBuffer::ansi();
    let color = if(isRed){TermColor::Red}else{TermColor::Green};
    t.set_color(ColorSpec::new().set_fg(Some(color)));
    t
}

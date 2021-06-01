pub mod engine;
pub mod utils;
pub use rand; // Re-export rand to let users use the correct version, compatible with wasm

#[cfg(any(feature = "visualization", feature = "visualization_wasm", doc))]
pub mod visualization;

#[cfg(any(feature = "visualization", feature = "visualization_wasm", doc))]
pub use bevy;

use rand::distributions::{Distribution, Uniform};


#[macro_export]
    ///step = simulation step number
    ///schedule
    ///agents
    ///states
    ///other parametes
macro_rules!  simulate{ 
    ($step:expr, $sch:expr, $ty:ty, $s:expr $(,$opt:expr)*) => {

    let n_step:u128 = $step;
    let mut schedule:Schedule<$ty> = $sch;
    println!("Num of steps {}", n_step);

    $(
        println!("Option received. {}", $opt);
    )*


    let start = std::time::Instant::now();
    for _ in 0..n_step{
        schedule.step(&mut $s);
    }

    let run_duration = start.elapsed();

    println!("Time elapsed in testing schedule is: {:?}", run_duration);
    println!("({:?}) Total Step:{}\nStep for seconds: {:?}",
    stringify!($ty),
    schedule.step,
    schedule.step as f64 /(run_duration.as_nanos() as f64 * 1e-9)

        );
 
    //Schedule don't use copy
    $sch = schedule; 
    };
}

#[macro_export]
///param type, min, max, num of param to gen
macro_rules! gen_param {
    ( $type:ty, $min:expr, $max:expr, $n:expr) => {{
        
        let minimum: $type;
        let maximum: $type;
        minimum = $min;
        maximum = $max;

        let (minimum, maximum) = if minimum > maximum {
            (maximum, minimum)
        }
        else if minimum == maximum {
            (minimum, maximum + 1 as $type)
        } else { (minimum, maximum)};

        let between = Uniform::from(minimum..maximum);
        let mut rng = rand::thread_rng();
        let dist:Vec<$type> = between.sample_iter(&mut rng).take($n).collect();

        dist
    }};

    //gen a single value
    (  $type:ty, $min:expr, $max:expr) => {{
        let minimum: $type;
        let maximum: $type;
        minimum = $min;
        maximum = $max;

        let (minimum, maximum) = if minimum > maximum {
            (maximum, minimum)
        }
        else if minimum == maximum {
            (minimum, maximum + 1 as $type)
        } else { (minimum, maximum)};

        //let between = Uniform::from(minimum..maximum);
        let mut rng = rand::thread_rng();

        rng.gen_range(minimum..maximum)

        
    }}
}



#[macro_export]
macro_rules! build_dataframe {
    ($name:ident $(,$element: ident: $ty: ty)*) => {
        #[derive(Debug)]        
        struct $name {
            pub $($element: Vec<$ty>),*
        }

        impl $name {
            pub fn init() -> $name{
                $name{ $(
                    $element: Vec::new(),
                )* }
            }
        }
    };

    ($name:ident, [input $($element: ident: $ty: ty),*] {output $($element2: ident: $ty2: ty),*}) => {
        #[derive(Debug)]        
        struct $name {
            pub $($element: Vec<$ty>),*
        }

        impl $name {
            pub fn init() -> $name{
                $name{ $(
                    $element: Vec::new(),
                )* }
            }
        }
    };
}

///WORK IN PROGRESS, DONT USE IT
#[macro_export]
    ///step = simulation step number
    ///schedule
    ///agents
    ///states
    ///sim param
macro_rules! explore {
    ($nstep: expr, $sch:expr, $agent_ty:ty, $s:expr, $rep_conf:expr $(,$param:ident: $ty: ty)*) => {{
        //typecheck
        let _rep_conf = $rep_conf as usize;
        let __nstep = $nstep as u128;

        let mut schedule:Schedule<$agent_ty> = $sch;
        println!("Calculate number of configuration");
        let mut n_conf:usize = 1;
        $( n_conf *= $param.len(); )*

        println!("n_conf {}", n_conf);

        build_dataframe!(DataFrame $( , $param:$ty )*);
        let mut conf  = DataFrame::init();
        //Build config
        $(
            let num_conf_per_val = n_conf / $param.len();
            
            for el in &$param{
                for i in 0..num_conf_per_val{
                    conf.$param.push(*el);
                }
            }

            drop($param);
        )*

        //load dataframe
        for i in 0..n_conf{
            //setting i-th run
            
            $(
                $s.$param = conf.$param[i]; 
            )*
            

            
            for j in 0..$rep_conf{
                $s.init(&mut schedule);

                println!("-----\nCONF {}, rep {}", i, j+1);
                $(
                    println!("{}: {}", stringify!($s.$param), $s.$param); 
                )* 
                simulate!($nstep, schedule, $agent_ty, $s);
                schedule.step  = 0;
            }
            //a[] = call output(state) function
            //add to dataframe a[]
            //return dataframe
        } 

    }}
}
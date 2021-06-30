pub mod engine;
pub mod utils;
pub use rand; // Re-export rand to let users use the correct version, compatible with wasm

#[cfg(any(feature = "visualization", feature = "visualization_wasm", doc))]
pub mod visualization;

#[cfg(any(feature = "visualization", feature = "visualization_wasm", doc))]
pub use bevy;

pub use rand::distributions::{Distribution, Uniform};

pub use csv::Writer;
use std::error::Error;
pub use std::time::Duration;
pub use rayon::prelude::*;

#[macro_use]
mod no_exported{
    #[macro_export]
    macro_rules! replace_expr {
        ($_t:tt $sub:expr) => {$sub};
    }
    
    #[macro_export]
    macro_rules! count_tts {
        ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
    }
}

///Create a csv file with the experiment results
///"DataFrame" trait allow the function to know field names and
///params list + output list for each configuration runned
pub fn export_dataframe<A: DataFrame>(
    name: &str,
    dataframe: &Vec<A>,
) -> Result<(), Box<dyn Error>> {
    let csv_name = format!("{}.csv", name);
    let mut wtr = Writer::from_path(csv_name).unwrap();

    //define column name
    wtr.write_record(A::field_names())?;

    for row in dataframe {
        wtr.serialize(row.to_string())?;
    }

    Ok(())
}

///Trait implemented dynamically for our dataframe struct.
///We use it into "export_dataframe" function
pub trait DataFrame {
    fn field_names() -> &'static [&'static str];
    fn to_string(&self) -> Vec<String>;
}

#[macro_export]
///step = simulation step number
///schedule
///agents
///states
///other parametes
macro_rules! simulate {
    ($step:expr, $s:expr) => {{
        let mut state = Box::new($s.as_state());
        let n_step: u128 = $step;
        let mut schedule: Schedule = Schedule::new();

        state.init(&mut schedule);

        println!("Num of steps {}", n_step);

        let start = std::time::Instant::now();
        for _ in 0..n_step {
            schedule.step(&mut state);
        }

        let run_duration = start.elapsed();

        println!("Time elapsed in testing schedule is: {:?}", run_duration);
        println!(
            "Total Step:{}\nStep for seconds: {:?}",
            schedule.step,
            schedule.step as f64 / (run_duration.as_nanos() as f64 * 1e-9)
        );

        (
            run_duration,
            schedule.step as f64 / (run_duration.as_nanos() as f64 * 1e-9),
        )
    }};

    ($step:expr, $s:expr, $reps:expr) => {{
        let mut results: Vec<(Duration, f64)> = Vec::new();
        for i in 0..$reps {
            println!("------\nRun {}", i + 1);
            results.push(simulate!($step, $s));
        }

        results
    }};
}

///Generate parameter values using a Uniform Distribution
///Params: Type, Min, Max and number of samples
///n_samples is optional, can be omitted if you want a single sample
#[macro_export]
macro_rules! gen_param {
    ( $type:ty, $min:expr, $max:expr, $n:expr) => {{
        let minimum: $type;
        let maximum: $type;
        minimum = $min;
        maximum = $max;
        let mut n = $n as usize;

        //Check range parameters to avoid error with Distribution
        let (minimum, maximum) = if minimum > maximum {
            (maximum, minimum)
        } else if minimum == maximum {
            (minimum, maximum + 1 as $type)
        } else {
            (minimum, maximum)
        };

        if n == 0 {
            n = 1;
        }

        let between = Uniform::from(minimum..maximum);
        let mut rng = rand::thread_rng();
        let dist: Vec<$type> = between.sample_iter(&mut rng).take($n).collect();

        dist
    }};

    //gen a single value
    (  $type:ty, $min:expr, $max:expr) => {{
        gen_param!($type, $min, $max, 1)
    }};
}

#[macro_export]
macro_rules! build_dataframe {
    //Dataframe with input and output parameters
    ($name:ident, input {$($input: ident: $input_ty: ty)*,}, output [$($output: ident: $output_ty: ty)*]) => {

        #[derive(Debug)]
        struct $name {
            pub conf_num: u128,
            pub conf_rep: u128,
            $(pub $input: $input_ty,)*
            $(pub $output: $output_ty,)*
            pub run_duration: Duration,
            pub step_per_sec: f64

        }

        impl DataFrame for $name{
            fn field_names() -> &'static [&'static str] {
                static NAMES: &'static [&'static str] = &["FrameRow", "Run", $(stringify!($input),)* $(stringify!($output),)*  "Run Duration", "Step per sec"];
                NAMES
            }

            fn to_string(&self) -> Vec<String> {
                let mut v: Vec<String> = Vec::new();
                v.push(self.conf_num.to_string());
                v.push(self.conf_rep.to_string());
                $(
                    v.push(self.$input.to_string());
                )*
                $(
                    v.push(self.$output.to_string());
                )*
                v.push(format!("{:?}", self.run_duration));
                v.push(self.step_per_sec.to_string());
                v
            }

        }

        impl $name {
            pub fn new( conf_num: u128, conf_rep: u128 $(, $input: $input_ty)* $(, $output: $output_ty)*, run_duration: Duration, step_per_sec: f64) -> $name{

                $name {
                    conf_num,
                    conf_rep,
                    $(
                        $input,
                    )*
                    $(
                        $output,
                    )*
                    run_duration,
                    step_per_sec
                }
            }
        }
    };

    //Dataframe with only input parameters
    ($name:ident $(, $element: ident: $input_ty: ty)*) => {
        build_dataframe!($name, input{$($element: $input_ty)*,}, output[]);
    };
}


///Brute force parameter exploration
#[macro_export]
///step = simulation step number,
///schedule,
///states,
///input{input: tipo},
///output[output: tipo]
macro_rules! explore {
    //exploration with explicit output parameters
    ($nstep: expr, $s:expr, $rep_conf:expr, input {$($input:ident: $input_ty: ty )*}, output [$($output:ident: $output_ty: ty )*]) => {{
        //typecheck
        let _rep_conf = $rep_conf as usize;
        let __nstep = $nstep as u128;

        println!("Calculate number of configuration");
        let mut n_conf:usize = 1;
        $( n_conf *= $input.len(); )*

        println!("n_conf {}", n_conf);

        build_dataframe!(FrameRow, input {$( $input:$input_ty)*, }, output[ $( $output:$output_ty )*]);


        //Cartesian product with variadics, to build a table with all parameter combinations
        //They are of different type, so i have to work with indexes
        let mut dataframe: Vec<FrameRow>  = Vec::new();
        let param_len = count_tts!($($input)*);
        
        let mut config_table_index:Vec<Vec<usize>> = (0..param_len).into_par_iter().map(|index_param|{
            let mut row:Vec<usize> = Vec::with_capacity(n_conf);
            let mut rep = n_conf;
            let mut input_size:usize = 0;
            {
                let mut i = 0;
                $(
                    if index_param >= i {
                        rep /= $input.len();
                        if index_param == i {
                            input_size = $input.len();
                        }
                        i+=1;
                    }                 
                )*
            }

            let mut i = 0;
            for _ in 0..n_conf{

                for _ in 0..rep{          
                        row.push(i);
                }
                i = (i + 1) % input_size;
            }
            row
        })
        .collect();

        for i in 0..n_conf{     
            let mut row_count = 0;
            $(
                $s.$input = $input[config_table_index[row_count][i]];
                row_count+=1;
            )*

            println!("-----\nCONF {}", i);
            $(
                println!("{}: {}", stringify!($s.$input), $s.$input);
            )*

            for j in 0..$rep_conf{
                println!("------\nRun {}", j+1);
                let result = simulate!($nstep, $s);
                dataframe.push( FrameRow::new(i as u128, j + 1 as u128, $($s.$input,)* $($s.$output,)* result.0, result.1));
            }

        }
        dataframe
    }};

    //exploration taking default output: total time and step per second
    ($nstep: expr, $s:expr, $rep_conf:expr, input {$($input:ident: $input_ty: ty )*}) => {
        explore!($nstep, $s, $rep_conf, input {$($input: $input_ty)*}, output [])
    }

}

#[macro_export]
macro_rules! explore_parallel {
    ($nstep: expr, $rep_conf:expr, $state_name:ty, param ($($parameter:expr,)*) , input {$($input:ident: $input_ty: ty )*}, output [$($output:ident: $output_ty: ty )*]) => {{

        //typecheck
        let _rep_conf = $rep_conf as usize;
        let _nstep = $nstep as u128;
        
        println!("Calculate number of configuration");
        let mut n_conf:usize = 1;
        $( n_conf *= $input.len(); )*
        println!("n_conf {}", n_conf);
        
        build_dataframe!(FrameRow, input {$( $input:$input_ty)*, }, output[ $( $output:$output_ty )*]);
        
        //let mut dataframe: Vec<FrameRow> = Vec::with_capacity(n_conf * $rep_conf);

        //---

        let param_len = count_tts!($($input)*);  
        let mut config_table_index:Vec<Vec<usize>> = (0..param_len).into_par_iter().map(|index_param|{
            let mut row:Vec<usize> = Vec::with_capacity(n_conf);
            let mut rep = n_conf;
            let mut input_size:usize = 0;
            {
                let mut i = 0;
                $(
                    if index_param >= i {
                        rep /= $input.len();
                        if index_param == i {
                            input_size = $input.len();
                        }
                        i+=1;
                    }                 
                )*

            }

            let mut i = 0;
            for _ in 0..n_conf{

                for _ in 0..rep{          
                        row.push(i);
                }
                i = (i + 1) % input_size;
            }
            row
        })
        .collect();

        //---

        let dataframe: Vec<FrameRow> = (0..n_conf*$rep_conf).into_par_iter().map( |run| {
            let i  = run / $rep_conf;
            let mut state = <$state_name>::new( $( $parameter ),*);
           

            let mut row_count = 0;
            $(
                state.$input = $input[config_table_index[row_count][i]];
                row_count+=1;
            )*

            let result = simulate!($nstep, state);

            println!("conf {}, rep {}, run {}", i, run / n_conf, run);
            FrameRow::new(i as u128, (run / n_conf) as u128, $(state.$input,)* $(state.$output,)* result.0, result.1)
        })
        .collect();


/*         for i in 0..n_conf{
            $(
                $s.$input = $input[i / (n_conf / $input.len())];
            )*
            println!("-----\nCONF {}", i);
            $(
                println!("{}: {}", stringify!($s.$input), $s.$input);
            )*
            for j in 0..$rep_conf{
                println!("------\nRun {}", j+1);
                let result = simulate!($nstep, $sch, $s);
                dataframe.push( FrameRow::new(i as u128, j + 1 as u128, $($input[i / (n_conf / $input.len())],)* $($s.$output,)* result.0, result.1));
            }
        } */
        dataframe
    }};

    //exploration taking default output and no state constructor: total time and step per second
    ($nstep: expr, $rep_conf:expr, $state_name:ty, input {$($input:ident: $input_ty: ty )*}) => {
            explore_parallel!($nstep, $rep_conf, $state_name, param (), input { $($input: $input_ty)*}, output [])
    };

    //exploration taking default output: total time and step per second
    ($nstep: expr, $rep_conf:expr, $state_name:ty, param ($($parameter:expr,)*), input {$($input:ident: $input_ty: ty )*}) => {
            explore_parallel!($nstep, $rep_conf, $state_name, param ($($parameter,)*), input { $($input: $input_ty)*}, output [])
    }; 

    //exploration with no state params constructor
    ($nstep: expr, $rep_conf:expr, $state_name:ty, input {$($input:ident: $input_ty: ty )*}, output [$($output:ident: $output_ty: ty )*]) => {
            explore_parallel!($nstep, $rep_conf, $state_name, param (), input { $($input: $input_ty)*}, output [])
    };

}

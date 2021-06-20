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
    ($step:expr, $sch:expr, $s:expr) => {{
        let mut state = Box::new($s.as_state());
        let n_step: u128 = $step;
        let schedule: &Schedule = &$sch;

        state.init(&mut $sch);

        println!("Num of steps {}", n_step);

        let start = std::time::Instant::now();
        for _ in 0..n_step {
            $sch.step(&mut state);
        }

        let run_duration = start.elapsed();

        println!("Time elapsed in testing schedule is: {:?}", run_duration);
        println!(
            "Total Step:{}\nStep for seconds: {:?}",
            $sch.step,
            $sch.step as f64 / (run_duration.as_nanos() as f64 * 1e-9)
        );

        (
            run_duration,
            $sch.step as f64 / (run_duration.as_nanos() as f64 * 1e-9),
        )
    }};

    ($step:expr, $sch:expr, $s:expr, $reps:expr) => {{
        let mut results: Vec<(Duration, f64)> = Vec::new();
        for i in 0..$reps {
            println!("------\nRun {}", i + 1);
            results.push(simulate!($step, $sch, $s));
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
                static NAMES: &'static [&'static str] = &["Configuration", "Run", $(stringify!($input),)* $(stringify!($output),)*  "Run Duration", "Step per sec"];
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
    ($nstep: expr, $sch:expr, $s:expr, $rep_conf:expr, input {$($input:ident: $input_ty: ty )*}, output [$($output:ident: $output_ty: ty )*]) => {{
        //typecheck
        let _rep_conf = $rep_conf as usize;
        let __nstep = $nstep as u128;

        let _schedule:&Schedule = &$sch;
        println!("Calculate number of configuration");
        let mut n_conf:usize = 1;
        $( n_conf *= $input.len(); )*

        println!("n_conf {}", n_conf);

        build_dataframe!(Configuration, input {$( $input:$input_ty)*, }, output[ $( $output:$output_ty )*]);

        let mut dataframe: Vec<Configuration>  = Vec::new();

        for i in 0..n_conf{

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
                dataframe.push( Configuration::new(i as u128, j + 1 as u128, $($input[i / (n_conf / $input.len())],)* $($s.$output,)* result.0, result.1));
            }

        }
        dataframe
    }};

    //exploration taking default output: total time and step per second
    ($nstep: expr, $sch:expr, $agent_ty:ty, $s:expr, $rep_conf:expr, $($input:ident: $input_ty: ty )*) => {
        explore!($nstep, $sch, $agent_ty, $s, $rep_conf, input { $($input: $input_ty)*}, output [])
    }
}

#[macro_export]
macro_rules! explore_parallel {
    ($nstep: expr, $rep_conf:expr, $state_name:ty, param ($($parameter:expr,)*) , input {$($input:ident: $input_ty: ty )*}, output [$($output:ident: $output_ty: ty )*]) => {{

        //typecheck
        let _rep_conf = $rep_conf as usize;
        let _nstep = $nstep as u128;
        //let _schedule:&Schedule = &$sch;
        
        println!("Calculate number of configuration");
        let mut n_conf:usize = 1;
        $( n_conf *= $input.len(); )*
        println!("n_conf {}", n_conf);
        
        build_dataframe!(Configuration, input {$( $input:$input_ty)*, }, output[ $( $output:$output_ty )*]);
        //let mut dataframe: Vec<Configuration> = Vec::with_capacity(n_conf * $rep_conf);

        let dataframe: Vec<Configuration> = (0..n_conf*$rep_conf).into_par_iter().map( |run| {
            let i  = run / $rep_conf;
            let mut schedule  = Schedule::new();
            let mut state = <$state_name>::new( $( $parameter ),*);
            $(
                state.$input = $input[i / (n_conf / $input.len())];
            )*

         /*    println!("-----\nCONF {}", i);
            $(
                println!("{}: {}", stringify!(state.$input), state.$input);
            )*
 */ 
            state.init(&mut schedule);
            let result = simulate!($nstep, schedule, state);

            println!("conf {}, rep {}, run {}", i, run / n_conf, run);
            Configuration::new(i as u128, (run / n_conf) as u128, $($input[i / (n_conf / $input.len())],)* $(state.$output,)* result.0, result.1)
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
                dataframe.push( Configuration::new(i as u128, j + 1 as u128, $($input[i / (n_conf / $input.len())],)* $($s.$output,)* result.0, result.1));
            }
        } */
        dataframe
    }};

/*     //exploration taking default output: total time and step per second
    ($nstep: expr, $sch:expr, $agent_ty:ty, $s:expr, $rep_conf:expr, $($input:ident: $input_ty: ty )*) => {
            explore!($nstep, $sch, $agent_ty, $s, $rep_conf, input { $($input: $input_ty)*}, output [])
    } */

}

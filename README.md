# SPLITS
Satisfiability Parallelism Leveraging Ideal Tree Splits

# Usage
In order to run SPLITS, just do `./splits -c config.cfg` where `config.cfg` is a valid config. See Configuration Options below for details, and `examples/` for some example configurations. Besides setting up the config for your use case, if you want to use `cadical` or `maxcdcl`, just ensure that in the scripts `cadical_wrapper.py` or `maxcdcl_wrapper.py` the `command` variable matches the location of `cadical` or `maxcdcl` on your system. Also, make sure the tracked metrics agree. 

TODO: Add all metrics, and make the usage of "packaged" wrappers more seamless.

# Configuration Options
- **variables**: The set of variables to split on. These must be positive integers.
- **comparator (optional)**: Whether to take the (min of max) or (max of min) of nodes in the tree. This must be either 'minmax' or 'maxmin'. By default, 'minmax' is used.
- **timeout (optional)**: The timeout in seconds for vertices in the tree during generation. This must be a positive integer number. By default, it is 600 seconds.
- **solver**: The location of the solver to be ran. It must be marked executable. See below for proper configuration details.
- **(w)cnf**: The location of the (w)cnf file.
- **output dir (optional)**: The directory that SPLITS will leave its outputs in. By default it is 'splits_output_directory'
- **tmp dir (optional)**: The directory that SPLITS will do work in. It will be cleaned up at the end of execution if it goes normally. By default it is 'splits_working_directory'
- **evaluation metric**: The metric by which vertices should be evaluated. This must appear in tracked metrics. See below for proper configuration details.
- **thread count (optional)**: The maximum number of threads to be used by SPLITS. **Warning: the default is the max on your system**
- **search depth (optional)**: How deep should each leaf search? The default is 1.
- **preserve cnf (optional)**: A boolean value whether the generated CNFs should be saved. This can be very costly on storage for large experiments. The default is false.
- **preserve logs (optional)**: A boolean value whether to store the logs of individual cube trials. The default is true.
- **cutoff proportion (optional)**: A float p between 0 and 1 representing the minimum "percentage improvement" the next layer must make to be considered valid. The default is 1 meaning any improvement is considered valid.
- **time proportion (optional)**L A float p > 0 representing the maximum decrease in time that a child cube can take. For example, if a cube takes t seconds, then its children can take at most p*t seconds. The default is 1, meaning that children are killed as soon as they take longer than their parents.
- **cutoff**: The value at which metrics should stop their search.

# The Interface of the Solver and Tracking Metrics
The solver must take two arguments as input `$1` is the (W)CNF file and `$2` is the log file where it should write its output.

The last two line of the solver's standard out should be of the form `SPLITS DATA \n {"metric1": num, "metric2": num, ... "metricn": num}`. Any metric that should be tracked in the logs should be placed in `tracked metrics`. Moreover, the metric used for comparison, the `evaluation metric`, should appear in `tracked metrics`. For example, if you wanted to track both 'ticks' and 'time' and make splitting decisions based off of seconds, the last line of the stdout of the solver would need to be: `{"time": 15.251, "ticks": 15816'}`. This is the default formatting of printing a python dictionary with the exception that double quotes must be used. Then, the config would need to contain:
```
tracked metrics: time ticks
evaluation metric: time
```
An example wrapper around cadical can be found in the `examples/` directory

## "time"
The only thing that is required for the ouput is that "time" must be tracked, even if it is not used as a cutoff metric. The reason for this is sometimes adding a variable to a cube can drastically degrade performance. Because of this, one should kill processes that take substantially longer
than previous iterations. When using `"time"` as a metric, I recommend using at most `1.0` as the `time proportion` setting in the config. Otherwise, some experimentation might be required.

## Responsibilities of the wrapper
The wrapper must do a few things in order to work properly. The most important thing is that when it recieves a `SIGTERM`, it forwards it (or some other signal) to kill the process that it spawns. Technically you don't *have* to do this, but if you don't you are going to have a bad time.

The other important thing is that when the solver finishes, it should block `SIGTERM` so it doesn't get interupted while writing logs. You also don't *have* to do this, but you could miss out on optimal splits if you don't.

For more details, read the README in `examples/`

# Note about Storage
Because programming is hard and I don't really know how syscalls work, the running storage footprint is fairly large. In particular, even if you don't store logs or CNFS, at each layer in the tree there is a chance they will only get cleaned up at the end. This is probably not an issue, but if you have very high `search depth` (> 4) and lots of `variables`, make sure you have at least a few gigabytes of storage availible just in case. I haven't measured this, and this is probably overkill, and I should probably fix this, but I'm putting a warning here for now.
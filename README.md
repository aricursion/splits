# SPLITS
Satisfiability Parallelism Leveraging Ideal Tree Splits

# Configuration Options
- **variables**: The set of variables to split on. These must be positive integers.
- **comparator (optional)**: Whether to take the (min of max) or (max of min) of nodes in the tree. This must be either 'minmax' or 'maxmin'. By default, 'minmax' is used.
- **timeout (optional)**: The timeout in seconds for vertices in the tree during generation. This must be a positive integer number. By default, it is 600 seconds.
- **solver**: The location of the solver to be ran. It must be marked executable. See below for proper configuration details.
- **cnf**: The location of the cnf file.
- **output dir (optional)**: The directory that SPLITS will leave its outputs in. By default it is 'splits_output_directory'
- **tmp dir (optional)**: The directory that SPLITS will do work in. It will be cleaned up at the end of execution if it goes normally. By default it is 'splits_working_directory'
- **tracked metrics**: The list of metrics to track. See below for proper configuration details.
- **evaluation metric**: The metric by which vertices should be evaluated. This must appear in tracked metrics. See below for proper configuration details.
- **thread count (optional)**: The maximum number of threads to be used by SPLITS. **Warning: the default is the max on your system**
- **search depth (optional)**: How deep should each leaf search? The default is 1.
- **preserve cnf (optional)**: A boolean value whether the generated CNFs should be saved. This can be very costly on storage for large experiments. The default is false.
- **preserve logs (optional)**: A boolean value whether to store the logs of individual cube trials. The default is true.
- **cutoff proportion (optional)**: A float p between 0 and 1 representing the minimum "percentage improvement" the next layer must make to be considered valid. The default is 1 meaning any improvement is considered valid.
- **cutoff**: The value at which metrics should stop their search.

# The Interface of the Solver
The last line of the solver's standard out should be of the form 'metric1: num, metric2: num, ... metricn: num'. Any metric that should be tracked in the logs should be placed in tracked metrics. Moreover, the metric used for comparison should appear in tracked metrics. For example, if you wanted to track both 'ticks' and 'seconds' and make splitting decisions based off of seconds, the last line of the stdout of the solver would need to be: 'seconds: 15.251, ticks: 15816'. Then, the config would need to contain:
```
tracked metrics: seconds ticks
evaluation metric: seconds
```
An example wrapper around cadical can be found in the examples directory

# Decisions to be made
- Currently the way that the solver function works is that if "time" is the metric then the solvers it spawns can run at most "time" seconds. Otherwise they can do at most `config.timeout` seconds. I'm not sure this is a good choice. Even if "time" isn't the main metric, it's unlikely you want to consider instances which take much longer than the previous one. But then should I require "time" to be tracked? 
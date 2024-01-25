# SPLITS
Satisfiability Parallelism Leveraging Ideal Tree Splits

# Configuration Options
- variables: The set of variables to split on. These must be positive integers.
- comparator (optional): Whether to take the min or max of nodes in the tree. This must be either 'min' or 'max'. By default, min is used.
- timeout (optional): The timeout in seconds for vertices in the tree during generation. This must be a positive integer number. By default, it is 600 seconds.
- solver: The location of the solver to be ran. It must be marked executable. See below for proper configuration details.
- cnf: The location of the cnf file.
- output dir (optional): The directory that SPLITS will leave its outputs in. By default it is 'splits_output_directory'
- tmp dir (optional): The directory that SPLITS will do work in. It will be cleaned up at the end of execution if it goes normally. By default it is 'splits_working_directory'
- tracked metrics: The list of metrics to track. See below for proper configuration details.
- evaluation metric: The metric by which vertices should be evaluated. This must appear in tracked metrics. See below for proper configuration details.

# The interface of the solver
The last line of the solver's standard out should be of the form 'metric1: num, metric2: num, ... metricn: num'. Any metric that should be tracked in the logs should be placed in tracked metrics. Moreover, the metric used for comparison should appear in tracked metrics. For example, if you wanted to track both 'ticks' and 'seconds' and make splitting decisions based off of seconds, the last line of the stdout of the solver would need to be: 'seconds: 15.251, ticks: 15816'. Then, the config would need to contain:
```
tracked metrics: seconds ticks
evaluation metric: seconds
```


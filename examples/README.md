# Configs
In this directory are two example configurations: `maximal.cfg` and `minimal.cfg`. 
The former is a config with every setting turned on, and the latter is the least number of settings for a functional configuration. 
In practice, you probably want somewhere in the middle. In particular, `multitree variables` is a pretty specialized setting that you likely don't want.

# Wrappers
In this directory there are wrappers for two solvers: `cadical` for SAT and `maxcdcl` for MaxSAT.
In general the way I intend them to be used is that `cadical_wrapper.sh` is given as the `solver` in the config and this calls `cadical_wrapper.py`.
I did this because formatting a command with arguments, such as `python3 cadical_wrapper.py`, as the "base command" could be inconsistent so I required the "base command" be a bash script. 
Technically, you don't need to have a further python wrapper, but I didn't want to do all of the signal handling and config parsing in bash.
In light of this, there is a `wrapper_template.py` script to get you started with adding your own solvers.

## Custom Solvers
This is a description of how I would go about adding a new solver - feel free to cut out the python script, or use another language, if you'd like. 
### The Bash script
You should have a bash script and the only thing it does is it calls the python script via `python3 my_wrapper.py $1 $2`. The arguments are important - `$1` will be the (w)cnf file and `$2` will be where SPLITS expects the logs to be written. **If your wrapper sends the output to stdout splits will not work**.

### The Python script
The python script is fairly straightforward. It's responsibility is to call the solver binary and wait for it to finish, or, when recieving a SIGTERM forward it to the solver. Then, if the solver terminates, parse the logs and write them to `$2`. An example can be seen in `wrapper_template.py`. The important parts are:

    1) `parse_metric` where you parse a field from your solver such as time, blocked clauses, etc. This must return a single float.
    2) `command` where you put the string corresponding to the command of the solver. For example `cadical` or `./my_solver`. It should not be given arguments as those are handled automatically.
    3) `metric_name` where you put what splits will recognize your metric as. This must match one of the `tracked metrics` in your config.

You are able to add more metrics as desired! Just repeat creating functions, names, and adding them to the dictionary as shown on line 59.


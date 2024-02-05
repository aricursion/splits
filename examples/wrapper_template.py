#!/usr/bin/python3
import subprocess
import sys


def parse_metric(output: str) -> float:
    NotImplemented


def run_cadical():
    log_file = open(sys.argv[2], "w")

    command = NotImplemented  # fill this in

    p = subprocess.Popen([command, sys.argv[1]], stdout=log_file)

    p.wait()
    # If the process completes, we should block SIGTERM so we can
    # finish writing the file and exit normally
    log_file.close()

    log_file = open(sys.argv[2], "r")

    output = log_file.read()

    log_file.close()
    log_file = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    log_file.write("SPLITS DATA\n")
    d = dict()

    # You can add other metrics in the exact same way
    metric_name = NotImplemented  # fill this in

    d[metric_name] = parse_metric(output)

    log_file.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_cadical()

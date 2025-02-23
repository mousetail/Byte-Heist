#!/bin/bash
if [ "$1" == "" ]; then
    echo "Expected a program to run"
    exit
fi

program="$(realpath "$1")"

$program &
PID=$!
restart () {
    kill -TERM $PID
    $program "${@:1}" &
    PID=$!
}
trap restart USR1
trap 'kill $!; exit' SIGTERM SIGHUP SIGINT

while true; do
    wait
done
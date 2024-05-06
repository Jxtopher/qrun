# Queue run

Manager task execution queues in parallel with a quenue depending on CPU/RAM load.

## Description

The backlog is stored in a single file, or a directory of files if you specify a backlog directory. ⚠ Backlog entries are read from these files and can also be updated because once a task is executed, it's automatically removed from the backlog to prevent duplicate execution.

## Example

* Execute all tasks listed in `protein-calculation.bl` with up to three processes in parallel (`-j`). The backlog can be updated on the fly if you add the tasks at the end.

```shell
qrun -b protein-calculation.bl -j 3
```

* `qrun` accepts a directory with a multiple of backlogs.

```shell
qrun -b /tmp/backlog
```

* `qrun` can work in demon mode (`-d`). When tasks are added to the backlog, the tasks will be detected and executed.

```shell
qrun -b /tmp/backlog -j 5 -d
```


* Other example of demon mode

```shell
bash -c "/usr/bin/qrun -b /var/spool/qrun -j 2 -d >/var/log/qrun.log 2>&1 &"
```

## See also

- Similare software `xjobs` (goto `man -s1 xjobs`)

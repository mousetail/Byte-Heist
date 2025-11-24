# Intro to Linux Namespaces and containerization

If you ask most programmers "how does Docker work?" they'll say something like "It's like a virtual machine but it
uses the same kernel". This is true, in a sense, but there is a lot more to it. 

The basic APIs Docker builds on is called a "Namespace". A namespace is not in itself a sandbox, it simply allows giving a process a different "view" of some aspects of the operating system. This has a lot of mundane uses, like:

- If a program requires a configuration file to exist in a certain filesystem but you can't place it there for some reason, you can give that program a "view" of the file system where that file is in a different location.
- You can give two programs a separate view of running processes, to avoid them noticing the other is running.
- Make a process think it's running as root even when it isn't

Each separate part of the operating system can be "unshared", giving it an isolated view. If you unshare each system separately, you can get something that feels a bit like a virtual machine. Not every part of the operating system has an unshare option though, so using these APIs alone will never give you a "sandbox"

## Types of namespaces

There are two very different kinds of namespaces in Linux.

### Root Namespaces 
"Root" namespaces can only be created by the root user. This is the system used by Docker, as it predates other options. The main disadvantage of root namespaces is, any APIs that are not (or can not) be unshared will have root privileges. 

For example, the system time, keyring, are not always namespaced so a container could theoretically change the system time if not blocked, something that would normally require root.

The "solution" to this is to entirely block access to any APIs which cannot be unshared. Docker uses two methods to block access to certain OS APIs. `SECCOMP` is a linux feature that allows blocking access to certain sys-calls. [Docker's default SECCOMP profile](https://github.com/moby/profiles/blob/main/seccomp/default.json) is a huge whitelist of allowed syscalls.

Notably, access to `clone` is blocked in Docker's `SECCOMP` which means you can not run any other containers inside docker by default.

In general, creating comprehensive `SECCOMP` profiles is very difficult since so many APIs allow privilege escalation, usually by manipulating mounts in some way. In 2002 Linux introduces the ["cababilities"](https://www.man7.org/linux/man-pages/man7/capabilities.7.html) API, intended to block or allow entire APIs at once rather than listing individual syscalls. However, they completely failed at this. At the time of writing, the majority of capabilities include at least one method to escalate privileges. One capability, `CAP_SYS_ADMIN` is extremely broad and encompasses a wide range of kernel subsystems. There has been a recent movement to split this capability off in more reasonable sections but the newly created segments on their own still allow privileged escalation in the majority of cases.

Still, despite the uselessness of the API for selecting which APIs to enable, disabling nearly all capabilities is a harmless extra security layer on top of `SECCOMP`.

### User Namespaces

A newer API that bypasses this entire problem is [user namespaces](https://www.man7.org/linux/man-pages/man7/user_namespaces.7.html). These APIs allow calling `unshare_*`/`clone` syscalls as a normal user, and the kernel will take care of ensuring the namespace can never do something a normal user could not.

Historically, this system was commonly disabled by more security conscious distributions because of various security vulnerabilities. However, the system seems to have finally earned it's trust, and is used by a huge veriety of software.

Browsers, for example, run each tab in a user namespace as an extra layer of defense in case an arbitrary code execution in their rendering or javascript engine occurs again. This is one reason browsers use so much RAM.

Other software which use user namespaces include Podman, a docker compatible container orchestration framework, and Flatpak.

Other than the obvious example of being able to use them if you are not root, user namespaces have a lot of other advantages. They can safely access a lot more APIs since as a user they can't do as much harm. This means it is possible to run them inside each-other without disabling important security systems.

## `cgroup`s vs namespaces

You might have heard of cgroups and wonder what their relation to namespaces is. They are often used together, but their use is slightly different.

A `cgroup` or "control group" is a group of processes that share some resource limits and can be killed as a group. They have a lot of uses outside of sandboxing but limiting the resource usage of sandboxes is usually desired, so tools that create namespaces, like Docker, also often put the sandboxed processes in a `cgroup`.

## Filling your namespace

If you just `unshare` some system, it will be empty. Almost all software has some dependencies, even if just dynamically linking to libc. Thus, you probably need to mount some files inside the namespace file system, connect the network interfaces to something if you want network access, or initialize the various systems in some way. 

Generally, this can be done with "ordinary" syscalls in the various systems that have some option to apply themselves to a different namespace, not anything namespace specific.

The process of mounting into a namespaces file system looks something like this:

- Run `__NR_open_tree` to create a virtual mount of a folder attached to the current process
- Use `setns` to switch to the container namespace
- Use `__NR_move_mount` to copy the virtual mount into the virtual file system.

There are many variations you can do, for example mount empty folders or read only mounts or whatever else. All of these require a significant amount of code per folder and could be slow. Some namespace systems create a `tempfs`, mount all folders there outside the namespace, then mount the entire `tempfs` inside the namespace to avoid a process needing to make too many namespace switches.

## `bwrap`, a low level namespacing tool

[`bwrap`](https://github.com/containers/bubblewrap) is a command line tool to manage namespaces by the same organization as Podman. It can make the process of actually filling your namespace with useful stuff a lot easier. We use it for Byte Heist, and it's also used by Flatpak and others.

`bwrap` is not like Docker, it does not have the concept of a "container" or building. It just has mounts and sharing. For example, you could run:

```bash
bwrap --ro-bind ./config /etc/nginx/conf.d nginx
```

to run `nginx` thinking `./config` is `/etc/nginx/conf.d`. In this case, we have not actually created a security boundary. `nginx` still has full access to everything we as a user have access to, it sees the filesystem differently but could in theory detect this happened and find the original file. 

To actually create a "secure" system, we need to unshare some APIs. You can unshare individual APIs like `--unshare-user` or more commonly, unshare all possible APIs with `--unshare-all`, then mount what you need using `--bind`.

This should now already be a secure container, but probably not enough to actually run anything. Unlike OCI containers, the process will not have a realistic view of an operating system. Inside a docker container, you can snoop around the file system and all the system directories and it will look much like a normal stripped down server, but if you create the namespace manually, it will be empty except for files you specifically add.

This makes actually getting software to run a bit of a challenge. Almost all software dynamically links to `libc` so you will need `--ro-bind /var/lib /var/lib --ro-bind /var/lib64 /var/lib64` to give it access to shared libraries. Most software will have many other dependencies to work properly and it can be a lot of effort to find them all and mount them properly.

You can then see a "container" as just a list of files that should be mounted in a namespace in order to create a realistic functional environment. An OCI container will typically include a lot more files than strictly needed, but in return almost all software will just work with no special configuration.

## Conclusion

In this article, I've shown at a birds eye view how various linux APIs can be combined to create different kinds of sandboxes. A summary of the systems needed is this:

- namespaces (user or root) to give the process a isolated, mutable view of kernel structures
- `SECCOMP` or capacities to limit the access to any APIs that can't be properly isolated
- `cgroup`s to allow managing the CPU and memory used by your sandbox, as well as allowing killing all processes inside the sandbox at once.
- Mounts to actually fill your sandbox with the necessary files to allow program to actually run in it.

Finally, I showed how to use `bwrap` to play with these features in a quick, low effort way.
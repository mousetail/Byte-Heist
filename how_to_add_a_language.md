# How to add a language

Adding a language can be a pretty frutrating experience because a improperly set up sandbox will often give unclear error messages and require a good amount of digging to work. In this guide, I'll explain what steps are nececairy and some tips on how to debug issues.

## Step 1: Deciding what language to add

We have a pretty small community on Byte Heist so even languages that might do well on [code.golf](https://code.golf) might not really get enough golfers to be worthwhile here. As a general rule we want at least
10 people who would golf in a language before it becomes worthwhile. If the leaderbaord is empty, that also discourages more people from submitting to the leaderboard.

Languages also need to meet certain technical requirements. A language should be able to complete all (easy or medium) challenges within the 3 second time limit. There is an option to add extra time for a
sufficiently popular language but that should be used sparingly.

Ideally, a language should be able to run relatively self-contained. A language that has a lot of dependencies will be hard to sandbox, plus people may abuse anything they have access to in the sandbox. For example, some languages make use of shell scripts and thus require access to codeutils which, if available, can lead to shorter solutions that do not use the language itself.

If you think a language would be a useful addition, talk about it on the discord. Get some feedback.

## Step 2: Find of create an ASDF plugin

We use the [ASDF Version Manager](https://asdf-vm.com/) to manage language versions. There are quite a lot of plugins publically available, [see a partial list](https://github.com/asdf-vm/asdf-plugins). If you can find an existing plugin, this is easy. Though in some cases it may be nececairy to fork and modify the plugin if it doesn't suit our needs.

If there is no plugin yet or the existing plugin is not suiable, you may need to create your own. [ASDF has a guide on how to do this](https://github.com/asdf-vm/asdf/blob/master/docs/plugins/create.md). You
might also want to look at examples of ASDF plugins created specifically for byte heist. [One for Vyxal 3](https://github.com/lyxal/vyxasd3f) and [one for TCC](https://github.com/mousetail/asdf-plugin-tcc).

If you are using the plugin template, you might notice it by default has a very stringent CI check. It does things like check for compatibility with macOS and various shells. It also checks if the plugin
creates an executable that can run via the asdf env. If you want to contribute the plugin back to the ASDF community, it could be nice to fix all those things but for Byte Heist, we don't care so
feel free to remove a lot of the CI rules. We don't even care if there is an executable in the path ASDF expects, all we care about that a folder exists with a self-contained version of all files related to the language.

Ideally, you'd bundle as many dependencies as possible. For example, if your language depends on python install python also inside the version folder. If many different languages rely on the same depenency and the dependency is reasonably forwards compatible it may not be nececairy but in general that should be preferred.

## Step 3: Creating an entry in `langs.rs`

In [langs.rs](./common/src/langs.rs), add an entry for your language. There are two types of languages:

- A compiled language specifies a `compile_command: &[]` to compile the source code, then a `run_command: &[]` to run it. 
- A interpreted language only specified a `run_command: &[]`.

First argument is the command to run, the rest are it's command line arguments. You can use the following substitutions:

- `${LANG_LOCATION}` Where the language version is mounted, eg. `/home/byte_heist/.asdf/installs/[lang name]/[version]
- `${FILE_LOCATION}` Where the source code is that should be run or compiled, eg `/code`
- `${OUTPUT_LOCATION}` Location where compiled artifacts should be placed. We cache these if you run multiple times with the same code so it's important to use this properly.

If you specify an extension, `file_location` will have this extension appended. Please only use the extension if your language actually requires the extension to run.

### Step 3.1: Finding a suitable logo

We use black and white logos for the language, logos should be in `.svg` format. Typically the easiest way is to simply edit the logo in your favorite text editor and fill all "fill=" lines and make them white. You might also need to remove some backgrounds to make the image not entirely white.

After you are done, make sure to run `svgo` on your logo to optimize and compress it.

## Step 4: Finding language dependencies

You should now be able to run your language. If you have followed the [local setup instructions](README.md), running `make restart-runner` should restart the lang runner with your new changes. You will also
need to restart the main server so your language and logo show up as an option in the tabs. Now try to run your language. You will usually see something resembling the following error:

```
failed to spawn /lang/bin/[your language name]: No such file or directory. No such file or directory. No such file or directory.
```

Unlike what the error would suggest, this does not mean that `/lang/bin/[your language name]` does not exist but rather that it has some dependency that is not available in the sandbox. Unfortunately the
error gives no clear sign of what dependency.

The best tool to figure this out is [strace](https://strace.io/). It can trace all system calls and figure out what files it tries to open. Unfortunatley, `strace` itself does not like to run in the sandbox,
but you can inspect what it would acces in normal circomstances by running it locally or inside the lang runner container. To add strace to the container, simply add `strace` to the `apt-install` line in the dockerfile, and then rebuild.

Now you can run `strace [lang executable]` and get a huge amount of output on every system call. Often it's helpful to use `strace [lang executable] 2> output.trace` to get a file for easier inspection. `strace` also supports various flags to filter what syscalls it traces. The calls most likely to cause issues are `openat`, the `exec*` family, `read`, etc. 

For each of these files, cross reference between the mounts in [run.rs](./lang-runner/src/run.rs). At the time of writing we mount `/lib`, `/lib64`, and `/usr/lib`, but it's best to check `run.rs` to get the most up do date list. Now filter these:

- If it's in a default mounted folder, should not be an issue
- If it's referencing something that should be in the asdf install folder, it could not be looking for it in the right place. You may need to set the correct environment variables or pass command line options to
tell the language where to seach. For instance, I often need to set `LD_LIBRARY_PATH` to make sure it looks for static libraries inside the `[your language name]/lib` folder.
- If it's something you can disable the language needing at all, eg. with a flag, try and disable it. eg. we often use stripped down versions of standard libraries that have less dependencies.
- Otherwise, you may need to add a mount.

If you need to add a mount, add an entry to `extra_mounts: &["local folder","sandbox folder"]`. For any libraries or data, prefer mounting the whole folder. If it's an executable in `/bin` or `/usr/bin`, mount
only the executable itself because these folders have too many abusable programs. If an external program is invovled, this program may have it's own dependencies so you may need to repeat the process.

Note that files in `/usr/bin` or `/bin` etc. are often symlinks so you may need to mount their destination too. Or even better use env vars or command line options to coax the language from looking in the correct location immediately.

You can sometimes get creative with mounts, like mount the symlinks destination directly in `/usr/bin` where this would normally be a symlink. This does not always work if it has dependencies it expects in the same folder.

## Step 5: Pull request and publish

If you got the language working with it's dependencies, submit a PR and if enough people are interested in golfing in it we will add it to the site.
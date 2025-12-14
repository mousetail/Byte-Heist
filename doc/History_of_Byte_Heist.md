# History of Byte Heist

## Initial Idea

[You can read about this discussion in the code.golf discord if you are a member](https://discord.com/channels/756829356155731988/807207439190720553/1285916453018669156)

The idea for Byte Heist started in the code.golf discord server 18 September 2024 when people expressed frustration with Ararchy Golf's extremely outdates language versions, and the lack of good alternatives for time-limited or ad-hoc challenges.

![Message from Sisyphus](https://mousetail.github.io/byte-heist-blog-images/discord_message_original_sysyphus.png)

[Sysyphus-ppcg](https://byte-heist.com/user/23/sisyphus-ppcg) suggested we should "create out own Anagol" to deal with these issues, either as a separate site or as an extension to code.golf.

This was followed by a lot of people contributing ideas and frustrations with the currently available options, even if it's just Anagol with a more modern interface.

[Mousetail](https://byte-heist.com/user/1/mousetail) suggested wanting time limited solutions that would eventually be revealed, similar to the then recently defunct week.golf. This suggestion got 3 upvotes.

[Sysyphus-ppcg](https://byte-heist.com/user/23/sisyphus-ppcg) suggested allowing scripting the program verifier using something like Lua to more easily create quite variants. This suggestion got 5 upvotes.

At first, the idea was to maybe create a separate section inside code.golf, but as [Ducky Luuk](https://byte-heist.com/user/25/duckyluuk) noted, code.golf's monolithic architecture made this difficult, as well as leaving a lot of difficult to answer questions about how this would effect the leaderboards.

## Forming an identity

[Ducky Luuk](https://byte-heist.com/user/25/duckyluuk) and [Mousetail](https://byte-heist.com/user/1/mousetail) discussed further in DMs. Originally, the idea was to build it in Node.js as that was the language DuckyLuuk was most familiar with, and to do containerization using docker. The idea at this point, as summarized dby Natte was "like week-golf and anagol with code.golf quality"

There was controversy at the start about how challenges would be picked. Anarchy golf allowed anyone to submit a challenge, and many people really liked that low friction experience that allowed for quick fun challenges with no curation. On the other hand, other people where afraid duplicates of holes on code.golf could create spoilers. In the end, there was a vote with 11 people in favor of curation and 3 against.

Another early point of contention was language restricted challenges. On code.golf, each hole is solvable in every language, and it's a collective effort to try to fill in the entire grid. This is fun and something people wanted to encourage, but fundamentally restricted source challenges might be impossible in some languages. The question then is: Should be preemptively remove the option to try and solve a challenge in a language which we think it would be impossible to solve in, and keep the community effort to fill in the parts of the grid thought to be possible, or keep the option open in case we are wrong?

A big fear at this stage was, given the flexibility of the judging, people would inject arbitrary requirements into their challenges depending on personal vendettas. [Code Golf Stack Exchange](https://codegolf.stackexchange.com/) has a frequent issues where challenge submitters will attempt to ban certain languages or classes of languages, either directly or by adding arbitrary requirements like a 0 exit code. This would hurt the consistency of challenges and also people just really didn't want the site to become another battle ground for the culture war that occasionally flared up on CGSE.

## Technical Foundations

[You can read back the original discussion in the discord here](https://discord.com/channels/1285973048092000388/1285976099708342313/1285977647850979400)

Any code challenge site needs the ability to run arbitrary user-submitted code securely. Originally, the idea was to use docker and spin up an OCI runtime for each submission. This technique was used by the late week.golf, but is both slow and requires a lot of storage to store all the images, especially if we wanted to store multiple versions for each language.

Bubbler suggested simply using the ATO or Piston API and not implementing our own at all, but this idea was rejected as it gave us no control over when languages would update.

[MJ](https://byte-heist.com/user/4/andriamanitra) suggested Bubblewrap, a relatively low level CLI for linux's name-spacing functionality. Bubbler suggested Isolate, a tool specifically for programming contests.

Unlike Docker, pure sandboxing tools require a seperate system for downloading and building specific language versions. [MJ](https://byte-heist.com/user/4/andriamanitra) suggested using ASDF, Mise, or Nix for that. While Nix has the greatest selection of language images, it was rejected as being to hard to use.

Bubbler summarized the discussion as follows:

> A train of thought:
> - we probably don't want the overhead of containers per submission
> - use asdf to install all supported versions of all langs (easy to setup for common langs, writing lang plug-ins doesn't seem terrible)
> - use isolate for sandboxing (if asdf shim is a problem, query executable via asdf where and run it directly)
> - for testing and to avoid accidents, maintain a single docker image that contains all of these and a server that takes run requests

[NicknamedTwice](https://byte-heist.com/user/13/nicknamedtwice) suggested replacing Isolate with Bubblewrap since it's able to run as non-root. Mousetail interviewed Pxeger, creator of Attempt This Online, and confirmed that Bubblewrap was a good choice, since it was used in the original version of ATO before being replaced by something custom.

From here on, Mousetail began experimenting with using Bubblewrap to run code. This was challenging at first because Mousetail has no idea what he was doing and Bubblewrap is fairly low level and does not give clear errors. It did work eventually.

## Invasion of Code Golf Stack Exchange

On the 13th of October 2024, news of the site "leaked" to CGSE, and several people who would become important later joined from there, including [Lyxal](https://byte-heist.com/user/2) the creator of the Vyxal language; [Jacob](https://byte-heist.com/user/6/jacob-lockwood), and [Madeline](https://byte-heist.com/user/3), the creator of TinyAPL.

The code.golf community and CGSE community have historically been at odds with each-other, especially in terms of their view on golf langs, though there is also a lot of overlap between the groups. The difference in perspective led to some discussions, but soon enough the groups become indistinguishable.

## Name

Finding a good name for the site was a challenge. One of the earliest suggestions was cata.golf by Bubbler, as kata is the opposite of ana and the site is sort of the opposite of anagolf. From there, KrausRous suggested ketamine.golf, a meme that took a bit too long to die.

Other suggestions based on variations of anarchy golf where monarchy.golf, demo.golf, and feudalism.golf.

There where also various golf related ideas like albatross, OpenGolf, bowling.sucks, or dont-use.java. golfing.city.

There where suggestions related to Mousetail's name, including mouse.golf, tail.golf, and golfta.il.

Eventually, the discussion went in the direction of saving bytes. Bubbler suggested "outgolfedby.one". In this vein, on the 6th of November 2024, [Jacob](https://byte-heist.com/user/6/jacob-lockwood) suggested "heist.golf". Which finally led into the rhyme "Byte Heist".

## Early Challenges

When the judging system was done around September 2024, a lot of people where excited to try ideas, and a large number of challenges where submitted in a short time. There was no distinction between beta and live challenges yet, it was a wild west. 

A few of the very early challenges have really stood the test of time:

- [Just Print](https://byte-heist.com/challenge/2/just-print/solve) by [MJ](https://byte-heist.com/user/4), which while easy is a very fun solve.
- [Double Quine](https://byte-heist.com/challenge/3/double-quine/solve) by [Jacob](https://byte-heist.com/user/6/jacob-lockwood) which uses the flexible scoring to create a quine variant that would be quite hard on any other site.
- [No Digits](https://byte-heist.com/challenge/6/no-digits/solve/python) by [AlephSquirrel](https://byte-heist.com/user/7), a classic fun restricted source problem.

Early challenges where all quite easy, intended to test site features, and overwhelmingly restricted source rather than pure code golf. A lot also had very limited test cases and relied more on hashing the test cases than solving the challenge.

### Example Code Controversy  

Byte Heist requires submitting some example code in Node.JS to verify if your judge works. However, this is sometimes a problem for restricted source heists. Often, the challenge is not in optimizing but just finding a trick that allows you to do anything at all. However, the public example code would give away the trick spoiling the challenge. Another issue was, for some types of restricted source, it was completely impossible to even solve in Node.JS.

Challenge authors first solved that by adding special cases in their judge that solutions submitted before a certain date would always pass regardless of if they passed the restricted source requirement. However, this made challenges impossible to edit as that would require re-testing the example code which could not pass after the date had lapsed.

Thus, mousetail added guidelines banning the use of "cheating" example code. This upset people who wanted to post fun puzzles. After some discussion, the authors and mousetail negotiated a compromise where "puzzle" style challenges could be posted as private challenges, however, this still left them uneditable and also made puzzles not count for scoring. A better way to deal with this situation is still pending.

### Radiation Resistant Integers and Radiation Resistant Integer Generators

[Radiation Resistant Integers](https://byte-heist.com/challenge/19/radiation-resistant-integers/view) and [Radiation Resistant Integer Generator](https://byte-heist.com/challenge/18/radiation-resistant-integers-generator/view) where some of the first difficult challenges on the site. The best score continued to reduce over months.

The judge for these challenges used hacks to determine the language of the submission and then wrap each submitted expression in a complete program. Only a few languages where supported, and it required submissions in a completely different format than other languages. Thus, according to the site rules at the time, these beta challenges could never become live and the people who put so much effort in could not get leaderboard points for their effort.

After the request of all submitters, we decided to real the solutions to these challenges in June 2025 without ever publishing them. [A blog was written about them](https://dev.to/dmrichwa/byte-heists-radiation-resistant-integers-post-mortem-408a). Unfortunately, despite the large amount of effort.

## Design Evolution

For the first 2-3 months, the site had no design and looked lie this:

![Challenge page on 18th of October](https://mousetail.github.io/byte-heist-blog-images/design_challenge_page_18_october.png)

![Home Page on November 2025](https://mousetail.github.io/byte-heist-blog-images/home_page_november_2024_image_by_duckyluuk.jpg)

[Jacob](https://byte-heist.com/user/6/jacob-lockwood) contributed the first redesign, a retro style which looked more like this, and a big improvement over the previous style. It looked like this:

![Design in February 2025](https://mousetail.github.io/byte-heist-blog-images/design_post_mortem_page_february_2025.png)

In May 2025, [Little Brownie](https://byte-heist.com/user/83/anishavelayudhan/achievements), a professional designer, volunteered to create an entirely new design that would look modern and professional. Mousetail initially released the new design in an unfinished state, and for several months, the site was full of placeholders.

![Design December 2025](https://mousetail.github.io/byte-heist-blog-images/design_december_2025.png)

## Moving towards a more community oriented curation process

A desire from the start was to not only allow an on site-way for creating challenges, but also to edit them and add test cases or other clean up without involvement of admins. However, we didn't want to risk changes invalidating large amounts of solutions. At the start of 2025, only admins where allowed to edit live challenges. So challenge authors had to often ask admins to fix things for them. Also, we wanted to avoid essentially re-implementing the entirety of GitHub.

In May 2025, Byte Heist introduced comments on the challenge view pages. This allowed for an on-site way to discuss problems with challenges. The feature was rarely used, since it was not very visible and Discord also existed for that kind of discussion.

In October 2025, the comments feature was extended to allow submitting a diff to a challenge as a comment. A change suggestion would be automatically merged when it got 3 upvotes (or rejected if it got 3 downvotes). You could only edit one of the judge or example code at the same time, so mass invalidating all submissions was difficult.

At this time, the discoverability of the comments and change suggestions was still poor so to get these suggestions actually merged usually required pinging people in the discord to get them to vote on the suggestions.

On the 2nd of December 2025, Byte Heist added the ability to vote on challenges. This would make selecting the beta challenges to promote easier and move the responsibility of curation even more in the hands of the community.


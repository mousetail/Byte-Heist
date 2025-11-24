# Scoring

We are experimenting with different scoring formulas, so the below is subject to change.

The scoring formula is designed to give more points to more difficult challenges (once with fewer solutions). 
You can get up to 260 points if you are the only submitter of a challenge in a given language, or the next closest solution is more than 350 bytes off.

On the other hand, the best solution could in theory get only 10 points if it is tied by more than 90% of all submitters.

## Mathematical Details

The current formula is as follows:
- You get 1 point for free regardless
- You get 10 points if there is no better solution than yours
- You get 1 point for each byte better than the top 10th percentile, up to a max of 50 points
- You get ½ point for each byte better than the 50th percentile, up to a max of 50 points
- You get ¼ point for each byte better than the 90th percentile up to a max of 50
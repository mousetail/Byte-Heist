export type LeaderboardEntry = {
  rank: number;
  author_avatar: string;
  author_name: string;
  author_id: number;
  points: number;
};

export type Challenge = {
  id: number;
  description: string;
  judge: string;
  name: string;
  example_code: string;
  category: "code-golf" | "restricted-source";
  status: "public" | "private" | "beta" | "draft";
  author: number;
  is_post_mortem: boolean;
  author_name: string;
  author_avatar: string;
  unit: string;
};

export type ScoreInfo = {
  rank: number;
  points: number;
  score: number;
};

export type Toast = {
  old_scores: ScoreInfo | undefined;
  new_scores: ScoreInfo;
};

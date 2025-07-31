interface Challenge {
    id: number;
    name: string;
    category: string;
    score?: number;
    description: string;
    isBeta?: boolean;
}

class HeistsFilter {
    private challenges: Challenge[] = [];
    private currentFilter: string = 'all';
    private searchTerm: string = '';

    constructor() {
        this.initializeChallenges();
        this.setupEventListeners();
    }

    private initializeChallenges(): void {
        // Get all challenge cards from the DOM
        const challengeCards = document.querySelectorAll('[data-challenge-card]');
        
        challengeCards.forEach(card => {
            const challenge: Challenge = {
                id: parseInt(card.getAttribute('data-challenge-id') || '0'),
                name: card.querySelector('[data-challenge-name]')?.textContent?.trim() || '',
                category: card.getAttribute('data-challenge-category') || '',
                score: this.parseScore(card.querySelector('[data-challenge-score]')?.textContent),
                description: card.querySelector('[data-challenge-description]')?.textContent?.trim() || '',
                isBeta: card.hasAttribute('data-challenge-beta')
            };
            this.challenges.push(challenge);
        });
    }

    private parseScore(scoreElement: Element | null): number | undefined {
        if (!scoreElement) return undefined;
        const text = scoreElement.textContent || '';
        const match = text.match(/(\d+)/);
        return match ? parseInt(match[1]) : undefined;
    }

    private setupEventListeners(): void {
        // Tab filtering
        const tabButtons = document.querySelectorAll('[data-filter-tab]');
        tabButtons.forEach(button => {
            button.addEventListener('click', (e) => {
                e.preventDefault();
                const filter = button.getAttribute('data-filter-tab') || 'all';
                this.setActiveTab(filter);
                this.filterChallenges();
            });
        });

        // Search functionality
        const searchInput = document.querySelector('[data-search-input]') as HTMLInputElement;
        if (searchInput) {
            searchInput.addEventListener('input', (e) => {
                this.searchTerm = (e.target as HTMLInputElement).value.toLowerCase();
                this.filterChallenges();
            });
        }

        // Difficulty filter
        const difficultySelect = document.querySelector('[data-difficulty-filter]') as HTMLSelectElement;
        if (difficultySelect) {
            difficultySelect.addEventListener('change', () => {
                this.filterChallenges();
            });
        }

        // Time filter
        const timeSelect = document.querySelector('[data-time-filter]') as HTMLSelectElement;
        if (timeSelect) {
            timeSelect.addEventListener('change', () => {
                this.filterChallenges();
            });
        }
    }

    private setActiveTab(filter: string): void {
        this.currentFilter = filter;
        
        // Update tab button states
        const tabButtons = document.querySelectorAll('[data-filter-tab]');
        tabButtons.forEach(button => {
            const buttonFilter = button.getAttribute('data-filter-tab') || 'all';
            if (buttonFilter === filter) {
                button.classList.remove('text-byte-brown-200', 'hover:text-white');
                button.classList.add('text-white', 'bg-green-900');
            } else {
                button.classList.remove('text-white', 'bg-green-900');
                button.classList.add('text-byte-brown-200', 'hover:text-white');
            }
        });
    }

    private filterChallenges(): void {
        const challengeCards = document.querySelectorAll('[data-challenge-card]');
        let visibleCount = 0;

        challengeCards.forEach((card, index) => {
            const challenge = this.challenges[index];
            if (!challenge) return;

            const matchesFilter = this.matchesFilter(challenge);
            const matchesSearch = this.matchesSearch(challenge);
            const matchesDifficulty = this.matchesDifficulty(challenge);
            const matchesTime = this.matchesTime(challenge);

            if (matchesFilter && matchesSearch && matchesDifficulty && matchesTime) {
                card.classList.remove('hidden');
                visibleCount++;
            } else {
                card.classList.add('hidden');
            }
        });

        this.updateEmptyState(visibleCount);
    }

    private matchesFilter(challenge: Challenge): boolean {
        switch (this.currentFilter) {
            case 'all':
                return true;
            case 'beta':
                return challenge.isBeta === true;
            case 'code-golf':
                return challenge.category === 'code-golf';
            case 'restricted-source':
                return challenge.category === 'restricted-source';
            case 'algorithms':
                return challenge.category === 'algorithms';
            case 'mathematical':
                return challenge.category === 'mathematical';
            case 'string-manipulation':
                return challenge.category === 'string-manipulation';
            case 'patterns':
                return challenge.category === 'patterns';
            default:
                return true;
        }
    }

    private matchesSearch(challenge: Challenge): boolean {
        if (!this.searchTerm) return true;
        
        return challenge.name.toLowerCase().includes(this.searchTerm) ||
               challenge.description.toLowerCase().includes(this.searchTerm);
    }

    private matchesDifficulty(challenge: Challenge): boolean {
        const difficultySelect = document.querySelector('[data-difficulty-filter]') as HTMLSelectElement;
        if (!difficultySelect || difficultySelect.value === 'all') return true;

        // This is a placeholder - you would need to add difficulty data to challenges
        // For now, we'll return true for all
        return true;
    }

    private matchesTime(challenge: Challenge): boolean {
        const timeSelect = document.querySelector('[data-time-filter]') as HTMLSelectElement;
        if (!timeSelect || timeSelect.value === 'all') return true;

        // This is a placeholder - you would need to add time data to challenges
        // For now, we'll return true for all
        return true;
    }

    private updateEmptyState(visibleCount: number): void {
        const emptyState = document.querySelector('[data-empty-state]');
        const grid = document.querySelector('[data-challenges-grid]');
        
        if (visibleCount === 0) {
            if (emptyState) {
                emptyState.classList.remove('hidden');
            }
            if (grid) {
                grid.classList.add('hidden');
            }
        } else {
            if (emptyState) {
                emptyState.classList.add('hidden');
            }
            if (grid) {
                grid.classList.remove('hidden');
            }
        }
    }
}

// Initialize the filter when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new HeistsFilter();
}); 
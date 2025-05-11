const setupVotingButton = (button: HTMLButtonElement) => {
    const is_upvote = button.value === 'true';
    const comment_id = +button.parentElement.querySelector<HTMLInputElement>('input[name=comment_id]').value;
    const other_button = button.parentElement.querySelector<HTMLButtonElement>(`button[name=is_upvote][value=${!is_upvote}]`);

    button.addEventListener('click', async (e)=>{
        e.preventDefault();
        button.disabled = true;

        let other_button_was_disabled = false;
        if (other_button.disabled) {
            other_button.disabled = false;
            other_button.textContent = other_button.textContent.replace(/\d+/, (e)=>`${+e-1}`);

            other_button_was_disabled = true;
        }

        button.textContent = button.textContent.replace(/\d+/, (e)=>`${+e+1}`);
        
        const res = await fetch(button.form.action, {
            method: 'POST',
            headers: {
                'Accept': 'Application/Json',
                'Content-Type': 'Application/Json',
            },
            redirect: 'manual',
            body: JSON.stringify({is_upvote: is_upvote, comment_id: comment_id})
        });

        if (res.status >= 400) {
            console.error(`Failed to submit vote: ${res.status}`);
            button.textContent = button.textContent.replace(/\d+/, (e)=>`${+e-1}`);
            button.disabled = false;

            other_button.disabled = other_button_was_disabled;
        }

    })
}

const setupComments = () => {
    const votingButtons = document.querySelectorAll('button[name=is_upvote]');
    votingButtons.forEach(setupVotingButton);
}

setupComments();
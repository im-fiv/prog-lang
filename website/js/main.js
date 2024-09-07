function redirectDownload() {
	window.location.href = 'https://github.com/im-fiv/prog-lang/archive/refs/heads/main.zip';
}

function redirectGitHub() {
	window.location.href = 'https://github.com/im-fiv/prog-lang';
}

async function executeCode() {
	let code = window.editor.getValue();
	let endpoint = `${window.origin}/execute`;

	let data;

	try {
		data = await fetch(endpoint, {
			body: code,
			method: "POST"
		});
	} catch (error) {
		console.error(error);
		return;
	}

	let output = await data.text();
	window.editor.setValue(output);
}
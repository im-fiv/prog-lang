import prog_lang from './monaco-prog-lang.js';
require.config({ paths: { vs: 'monaco-editor/min/vs' } });

require(['vs/editor/editor.main'], function() {
	monaco.languages.register({ id: 'prog-lang' });
	monaco.languages.setMonarchTokensProvider('prog-lang', prog_lang);

	window.editor = monaco.editor.create(document.getElementById('container'), {
		value: '("Hello, World!") -> print',
		language: 'prog-lang',
		theme: 'vs-dark'
	});
});
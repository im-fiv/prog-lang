export default {
	keywords: [
		'def', 'func', 'do', 'end', 'return', 'while',
		'break', 'continue', 'if', 'then', 'none',
		'self', 'class', 'extern'
	],

	typeKeywords: [
		'true', 'false', 'void'
	],

	intrinsics: [
		'print', 'import', 'input', 'raw_print', 'assert'
	],

	operators: [
		'=', '==', '-', '!', '+', '/', '*',
		'%', '!=', '&&', '||', '>=', '<=',
		'>', '<', '->', '=>', '='
	],

	symbols: /[=><!~?:&|+\-*\/\^%]+/,

	tokenizer: {
		root: [
			// identifiers and keywords
			[/[a-z_$][\w$]*/, {
				cases: {
					'@intrinsics': 'keyword',
					'@typeKeywords': 'keyword',
					'@keywords': 'keyword',
					'@default': 'identifier'
				}
			}],

			{ include: '@whitespace' },

			// type names
			[/[A-Z][\w\$]*/, 'type.identifier'],

			// operators
			[/[()]/, '@brackets'],
			[/@symbols/, {
				cases: {
					'@operators': 'operator',
					'@default': ''
				}
			}],

			// numbers
			[/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
			[/0[xX][0-9a-fA-F]+/, 'number.hex'],
			[/\d+/, 'number'],

			// delimeters
			[/[,.]/, 'delimiter'],

			// strings
			[/"([^"\\]|\\.)*$/, 'string.invalid' ],
			[/"/,  { token: 'string.quote', bracket: '@open', next: '@string' } ]
		],

		comment: [
			[/[^\/*]+/, 'comment'],
			[/\/\*/, 'comment', '@push'],
			["\\*/", 'comment', '@pop'],
			[/[\/*]/, 'comment']
		],

		string: [
			[/[^\\"]+/, 'string'],
			[/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }]
		],

		whitespace: [
			[/[ \t\r\n]+/, 'white'],
			[/\/\*/, 'comment', '@comment'],
			[/\/\/.*$/, 'comment'],
		]
	}
}
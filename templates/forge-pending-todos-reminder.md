{{#if todos}}
You have pending todo items:
{{#each todos}}
- [{{status}}] {{content}}
{{/each}}
{{/if}}
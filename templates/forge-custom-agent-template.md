{{#if agent}}{{agent.agent_name}}{{else}}Agent{{/if}}{{#if agent.description}} - {{agent.description}}{{/if}} Custom Agent Template

{{#if extensions}}
{{#with extensions}}
{{#if extension_stats}}
Extensions:
{{#each extension_stats}}
- {{extension}} ({{count}} files, {{percentage}}%)
{{/each}}
{{/if}}
{{/with}}
{{/if}}
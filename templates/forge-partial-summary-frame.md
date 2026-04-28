{{#if messages}}
# Context Summary

{{#each messages}}
## {{role}}

{{#each contents}}
{{#if Text}}{{this}}{{/if}}
{{/each}}

{{/each}}
{{/if}}

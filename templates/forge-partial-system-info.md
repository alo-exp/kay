<operating_system>{{env.os}}</operating_system>
<current_working_directory>{{env.cwd}}</current_working_directory>
<shell>{{env.shell}}</shell>
<home_directory>{{env.home}}</home_directory>
<current_time>{{current_time}}</current_time>
<workspace_extensions files="{{#each files}}{{path}}{{#unless @last}}, {{/unless}}{{/each}}">
import json, pathlib
root=pathlib.Path(__file__).resolve().parent/'schemas'
D='https://json-schema.org/draft/2020-12/schema'
V={'type':'string','const':'1.0.0'}
s=lambda **kw:{'type':'string',**kw}
arr=lambda item:{'type':'array','items':item}
obj=lambda props,required=None,extra=False:{'type':'object','additionalProperties':extra,'properties':props,'required':required or list(props)}
enum=lambda x:{'type':'string','enum':x.split(' ')}

schemas={}

cfg=obj({
 'schema_version':V,'project_id':s(minLength=1,pattern='^[a-z0-9][a-z0-9-_]*$'),
 'target':obj({'kind':enum('website screenshot folder'),'url':s(pattern='^https?://[^\\s]+$')},['kind'],False),
 'mode':enum('refresh redesign reimagine'),
 'capture_mode':enum('clean_room deep'),
 'scope':obj({'max_routes':{'type':'integer','minimum':1,'maximum':1000},'max_journeys':{'type':'integer','minimum':1,'maximum':100},'roles':arr(s())},['max_routes','max_journeys','roles']),
 'design_system':obj({'kind':enum('none local_repo package storybook tokens layout_kit'),'source':s(),'policy':enum('strict advisory')},['kind','policy']),
 'providers':obj({'rea':enum('disabled auto enabled'),'application_mcp':enum('disabled enabled')},['rea','application_mcp']),
 'backend':obj({'mode':enum('mock sandbox staging'),'allow_production_writes':{'type':'boolean','const':False}},['mode','allow_production_writes']),
  'model':obj({'use_craft_selected_backend':{'type':'boolean'}},['use_craft_selected_backend']),
 'execution':obj({
  'controller':enum('host_agent craft_runner'),
  'profile':enum('lean deep'),
  'allow_nested_builder':{'type':'boolean','const':False},
  'budgets':obj({
    'token_soft_limit':{'type':'integer','minimum':0},
    'max_mcp_calls':{'type':'integer','minimum':0},
    'max_rea_invocations':{'type':'integer','minimum':0},
    'max_critic_passes':{'type':'integer','minimum':0},
    'max_tool_result_chars':{'type':'integer','minimum':1024},
    'max_run_minutes':{'type':'integer','minimum':1},
    'no_progress_limit':{'type':'integer','minimum':1},
  },[],False)
 },['controller','profile','allow_nested_builder'],False),
 'privacy':obj({'model_egress':enum('local_only allow_remote_redacted allow_remote_explicit'),'export_raw_captures':{'type':'boolean'}},['model_egress','export_raw_captures']),
 'checks':obj({'max_refine_rounds':{'type':'integer','minimum':0,'maximum':20},'require_critical_scenarios':{'type':'boolean'}},['max_refine_rounds','require_critical_scenarios'])
})
cfg['allOf']=[
 {'if':{'properties':{'target':{'properties':{'kind':{'const':'website'}}}}},'then':{'properties':{'target':{'required':['kind','url']}}}},
 {'if':{'properties':{'execution':{'required':['controller'],'properties':{'controller':{'const':'host_agent'}}}},'required':['execution']},
  'then':{'properties':{'model':{'properties':{'use_craft_selected_backend':{'const':False}}}}}}
]
schemas['redesign-config.schema.json']=cfg

scope=obj({'target_origin':s(),'route':s(),'role':s(),'environment':s(),'app_revision':s()},[],False)
locator=obj({'artifact_path':s(),'line':{'type':'integer','minimum':1},'step_id':s(),'external_evidence_id':s()},[],False)
ev=obj({'id':s(pattern='^ev_[a-zA-Z0-9_-]+$'),'source':enum('craft rea app_mcp user source_repo test'),
 'source_operation':s(minLength=1),'subject':s(minLength=1),
 'kind':enum('ui behavior business_rule api design_system test user_context'),
 'status':enum('observed declared inferred unknown validated refuted'),
 'scope':scope,'locator':locator,'digest':s(),
 'observed_at':s(format='date-time'),'limitations':arr(s()),
 'sensitivity':enum('public internal restricted unknown'),'summary':s()},
 ['id','source','source_operation','subject','kind','status','scope','observed_at','limitations','sensitivity','summary'])
schemas['evidence-record.schema.json']=ev

cap=obj({'id':s(pattern='^cap_[a-zA-Z0-9_-]+$'),'title':s(minLength=1),'task':s(minLength=1),
 'actor_roles':arr(s()),'original_routes':arr(s()),'inputs':arr(s()),'actions':arr(s()),
 'outputs':arr(s()),'preconditions':arr(s()),'states':arr(s()),'rule_ids':arr(s()),
 'evidence_ids':arr(s(pattern='^ev_')),'criticality':enum('critical high normal low'),
 'coverage':enum('covered partial uncovered unknown'),'unknowns':arr(s()),'scenario_ids':arr(s())})
schemas['capability.schema.json']=cap

component=obj({'id':s(minLength=1),'package_name':s(minLength=1),'package_version':s(minLength=1),
 'import_path':s(minLength=1),'export_name':s(minLength=1),
 'props_schema':{'anyOf':[{'type':'object'},{'type':'null'}]},
 'variants':arr(s()),'tokens':arr(s()),'allowed':{'type':'boolean'},
 'usage':arr(s()),'never':arr(s()),'evidence_ids':arr(s())})
ds=obj({'schema_version':V,'project_id':s(minLength=1),'revision':{'type':'integer','minimum':0},
 'generated_at':s(format='date-time'),'policy':enum('strict advisory'),
 'source_ref':s(minLength=1),'source_digest':s(minLength=1),'components':arr(component),'tokens':{'type':'object'}},
 ['schema_version','project_id','revision','generated_at','policy','source_ref','source_digest','components','tokens'])
schemas['design-system-registry.schema.json']=ds

option=obj({'id':s(),'description':s(),'risks':arr(s())})
decision=obj({'id':s(minLength=1),'capability_ids':arr(s()),'problem':s(minLength=1),
 'options':{'type':'array','minItems':1,'items':option},'chosen_option_id':s(),
 'rationale':s(),'ux_rule_ids':arr(s()),'evidence_ids':arr(s()),
 'feasibility':enum('ui_only frontend_orchestration backend_change_required not_determined'),
 'dependencies':arr(s()),'status':enum('proposed approved rejected'),'approval_evidence_id':s()},
 ['id','capability_ids','problem','options','chosen_option_id','rationale','ux_rule_ids','evidence_ids','feasibility','dependencies','status'])
schemas['design-decision.schema.json']=decision

mapping=obj({'capability_id':s(pattern='^cap_'),'target_route':s(),'target_component':s(),
 'target_action':s(),'decision_id':s(),'scenario_ids':arr(s()),
 'exception_approved':{'type':'boolean'}},['capability_id','decision_id','scenario_ids','exception_approved'])
plan=obj({'schema_version':V,'project_id':s(minLength=1),'revision':{'type':'integer','minimum':0},'generated_at':s(format='date-time'),
 'mode':enum('refresh redesign reimagine'),'snapshot_id':s(minLength=1),
 'mappings':arr(mapping),'unknowns':arr(s()),'decisions':arr(decision)},
 ['schema_version','project_id','revision','generated_at','mode','snapshot_id','mappings','unknowns','decisions'])
schemas['transformation-plan.schema.json']=plan

check=obj({'id':s(minLength=1),'check_id':s(minLength=1),'status':enum('pass fail warn not_evaluated blocked'),
 'severity':enum('blocking major minor info'),'executed':{'type':'boolean'},'environment':s(),
 'tool_version':{'anyOf':[s(),{'type':'null'}]},'scenario_ids':arr(s()),
 'evidence_ids':arr(s()),'started_at':s(format='date-time'),
 'duration_ms':{'type':'integer','minimum':0},'message':s(),'limitations':arr(s())})
check['allOf']=[{'if':{'properties':{'executed':{'const':False}}},'then':{'properties':{'status':{'enum':['not_evaluated','blocked']}}}}]
schemas['check-result.schema.json']=check

sc=obj({'id':s(pattern='^scenario_'),'capability_id':s(pattern='^cap_'),'profile':s(),
 'fixture_id':s(),'given':arr(s()),'when':arr(s()),'then':arr(s()),
 'assertion_source':enum('observed declared user_approved'), 'evidence_ids':arr(s()),
 'safe_execution_target':enum('generated_mock sandbox staging'),'executable':{'type':'boolean'}})
schemas['scenario-oracle.schema.json']=sc

usage=obj({
 'schema_version':V,
 'run_id':s(minLength=1),
 'source':s(minLength=1),
 'accounting_quality':enum('provider_reported host_reported estimated unavailable'),
 'input_tokens_reported':{'anyOf':[{'type':'integer','minimum':0},{'type':'null'}]},
 'output_tokens_reported':{'anyOf':[{'type':'integer','minimum':0},{'type':'null'}]},
 'cached_input_tokens_reported':{'anyOf':[{'type':'integer','minimum':0},{'type':'null'}]},
 'tokens_estimated':{'anyOf':[{'type':'integer','minimum':0},{'type':'null'}]},
 'cost_amount':{'anyOf':[{'type':'number','minimum':0},{'type':'null'}]},
 'cost_currency':{'anyOf':[s(pattern='^[A-Z]{3}$'),{'type':'null'}]},
 'tool_calls':{'type':'integer','minimum':0},
 'captured_at':s(format='date-time'),
 'limitations':arr(s())
})
usage['allOf']=[
 {'if':{'properties':{'accounting_quality':{'const':'unavailable'}}},
  'then':{'properties':{
   'input_tokens_reported':{'type':'null'},
   'output_tokens_reported':{'type':'null'},
   'cached_input_tokens_reported':{'type':'null'},
   'tokens_estimated':{'type':'null'},
  }}},
 {'if':{'properties':{'cost_amount':{'type':'number'}}},
  'then':{'properties':{'cost_currency':{'type':'string'}}}}
]
schemas['usage-snapshot.schema.json']=usage

for name,doc in schemas.items():
 doc['$schema']=D;doc['$id']='https://schemas.redesign-anything.example/1.0.0/'+name
 doc['title']=name.split('.')[0]
 (root/name).write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Created',len(schemas),'JSON Schemas',', '.join(schemas))

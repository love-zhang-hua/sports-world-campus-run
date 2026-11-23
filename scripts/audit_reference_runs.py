"""Reproducible arithmetic audit of anonymised screenshot summaries, not physiology."""
from pathlib import Path
import csv,json,statistics

root=Path(__file__).resolve().parents[1]
rows=list(csv.DictReader((root/'docs/reference-runs.csv').open(encoding='utf-8-sig')))
out=[]
for r in rows:
    t,p,c,l,steps=(float(r[k]) for k in ['duration_s','pace_s_km','cadence_spm','stride_cm','steps'])
    # Assume nearest-unit display rounding only for the consistency screen.
    # This is NOT a statistical error model or instrument accuracy estimate.
    cadence_interval=(steps*60/(t+.5),steps*60/(t-.5))
    cadence_ok=cadence_interval[0]<=c+.5 and cadence_interval[1]>=c-.5
    pace_v=(1000/(p+.5),1000/(p-.5))
    gait_v=((c-.5)*(l-.5)/6000,(c+.5)*(l+.5)/6000)
    speed_ok=max(pace_v[0],gait_v[0])<=min(pace_v[1],gait_v[1])
    dist_ok=None;dist_delta=None
    if r['distance_km']:
        d=float(r['distance_km']);est=((t-.5)/(p+.5),(t+.5)/(p-.5))
        dist_ok=est[0]<=d+.005 and est[1]>=d-.005
        dist_delta=100*((t/p)/d-1)
    out.append(dict(group=r['group'],date=r['date'],
        cadence_from_steps_time=steps*60/t,cadence_delta_spm=steps*60/t-c,
        speed_identity_relative_error_pct=100*((c*l/6000)/(1000/p)-1),
        distance_relative_error_pct=dist_delta,
        cadence_rounding_intervals_overlap=cadence_ok,
        speed_rounding_intervals_overlap=speed_ok,
        displayed_distance_rounding_intervals_overlap=dist_ok))
groups={}
for group in ['A','B','all']:
    rr=[r for r in rows if group=='all' or r['group']==group]
    v=[1000/float(r['pace_s_km']) for r in rr];c=[float(r['cadence_spm']) for r in rr]
    vm,cm=statistics.mean(v),statistics.mean(c)
    slope=sum((a-vm)*(b-cm) for a,b in zip(v,c))/sum((a-vm)**2 for a in v)
    groups[group]=dict(records=len(rr),mean_cadence_spm=cm,descriptive_ols_slope_spm_per_m_s=slope)
result=dict(records=len(rows),people=2,rounding_assumption='Nearest 1s duration/pace, 1spm cadence, 1cm step length, 0.01km displayed distance; steps treated as integer count. Interval overlap is only arithmetic compatibility, not sensor validation.',
    rows=out,groups=groups,
    max_abs_cadence_delta_spm=max(abs(x['cadence_delta_spm']) for x in out),
    max_abs_speed_error_pct=max(abs(x['speed_identity_relative_error_pct']) for x in out),
    limitations='Repeated summaries from two runners, no controlled speed trial. Neither pooled nor within-person summary slopes establish a physiological speed response. Missing fields stay missing; no inference of measured power, calories or ascent.')
assert all(x['cadence_rounding_intervals_overlap'] and x['speed_rounding_intervals_overlap'] and x['displayed_distance_rounding_intervals_overlap'] is not False for x in out)
(root/'docs/reference-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['rows','groups']},ensure_ascii=False,indent=2))

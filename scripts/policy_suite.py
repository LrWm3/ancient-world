"""Strict, within-checkpoint policy contrasts. No fitting or seed selection."""
POLICIES = ('adaptive', 'fixed', 'food-maintenance', 'food-only')


def grouped(data):
    groups = {}
    for run in data['runs']:
        key = (run['seed'], run['tool_fraction'])
        group = groups.setdefault(key, {})
        policy = run['policy']
        if policy in group or policy not in POLICIES:
            raise ValueError('Duplicate or unknown policy')
        group[policy] = run
    for group in groups.values():
        if set(group) != set(POLICIES):
            raise ValueError('Incomplete policy factorial')
        base = group['adaptive']
        for run in group.values():
            if run['config'] != base['config'] or run['initial'] != base['initial']:
                raise ValueError('Unmatched checkpoint/configuration')
            if [b['closed'] for b in run['branches']] != [False, True]:
                raise ValueError('Missing or reordered mine branches')
            for branch in run['branches']:
                if len(branch['months']) != data['closure_months'] + data['recovery_months']:
                    raise ValueError('Incomplete monthly trajectory')
                if [m['month'] for m in branch['months']] != [m['month'] for m in base['branches'][0]['months']]:
                    raise ValueError('Unmatched clocks')
    return groups


def render(data):
    from summarize_coupling import sum_field, tool_multiplier
    groups = grouped(data)
    lines = ['# Maintenance ablation and policy evaluation', '',
             f"GPU: {data['gpu']}; backend: {data.get('backend')}; driver: {data.get('driver')}", '',
             'Matched checkpoints; finite inventory and unchanged coefficients. All four policies run on every seed/reserve combination. Differences below are first policy minus second.', '',
             '| Seed | Tool access | Closed mines | Contrast | Month | Settled population Δ | Harvest Δ kg cumulative | Unmet rations Δ kg cumulative | Tool multiplier Δ | Tools made Δ kg cumulative |',
             '|---|---|---|---|---:|---:|---:|---:|---:|---:|']
    for (seed, fraction), group in groups.items():
        for left, right in [('food-maintenance', 'adaptive'), ('food-only', 'adaptive'), ('food-maintenance', 'food-only')]:
            for idx, closed in enumerate((False, True)):
                a, b = [group[p]['branches'][idx]['months'] for p in (left, right)]
                for n in sorted({data['closure_months'], len(a)}):
                    population = sum_field(a[n-1], 'population')-sum_field(b[n-1], 'population')
                    harvest = sum(sum_field(m, 'production_kg')-sum_field(k, 'production_kg') for m,k in zip(a[:n], b[:n]))
                    unmet = lambda seq: sum(sum_field(m, 'ration_need_kg', 3)-sum_field(m, 'ration_eaten_kg', 3) for m in seq[:n])
                    made = sum_field(a[n-1], 'tool_made_kg')-sum_field(b[n-1], 'tool_made_kg')
                    tool = tool_multiplier(a[n-1])-tool_multiplier(b[n-1])
                    lines.append(f'| {seed} | {fraction:g} | {closed} | {left} − {right} | {n} | {population:.3f} | {harvest:.2f} | {unmet(a)-unmet(b):.2f} | {tool:.6f} | {made:.3f} |')
    lines += ['', '| Seed | Tool access | Closed mines | Maintenance: first differing labor month | Identical complete monthly observations |', '|---|---|---|---:|---:|']
    for (seed, fraction), group in groups.items():
        for idx, closed in enumerate((False, True)):
            a,b = [group[p]['branches'][idx]['months'] for p in ('food-maintenance','food-only')]
            first = next((i+1 for i,(m,n) in enumerate(zip(a,b)) if [(s['site'],s['labor_workers']) for s in m['sites']] != [(s['site'],s['labor_workers']) for s in n['sites']]), 'none')
            lines.append(f'| {seed} | {fraction:g} | {closed} | {first} | {sum(m==n for m,n in zip(a,b))}/{len(a)} |')
    months = [m for r in data['runs'] for b in r['branches'] for m in b['months']]
    for field in ('economy_residuals','source_residual','population_residual','ecology_relative_error','ecology_water_relative_error'):
        values = [abs(v) for m in months for v in (m[field] if isinstance(m[field],list) else [m[field]])]
        lines += ['', f'Maximum {field}: {max(values):.6e}.']
    lines += ['', f'{len(months)} monthly observations. Unmet rations are measured kg calorie equivalents; population and therefore ration demand can change. Later differences do not isolate every intermediate mechanism. Identical recorded observations do not prove all GPU state identical.',
              'These are legacy shared-resource economies at one terrain/ecology resolution, not empirical calibration or cross-hardware validation.', '']
    return '\n'.join(lines)

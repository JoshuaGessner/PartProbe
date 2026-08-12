# Test Data Strategy

> **Status:** In Review  
> **Last updated:** 2026-08-12
> **Related requirements:** REQ-NF-002, REQ-NF-005; TEST-001–TEST-012  
> **Related ADRs:** ADR-0002–ADR-0008  
> **Open questions:** Rights to shop-derived anonymized fixtures  
> **Dependencies:** Data classification policy  
> **Supersedes:** None

Use three clearly separated datasets:

1. **Public synthetic:** generated primitives and invented estimates committed to the repository.
2. **Licensed public:** redistribution rights and provenance recorded per fixture.
3. **Private shop validation:** encrypted/access-controlled outside the public repository; manifest IDs and aggregate results only where authorized.

Never assume deleting names anonymizes geometry. A part shape can itself be identifying technical data. Do not upload private fixtures to CI or external analysis services. Each fixture has a source/license/classification, hash, expected results/tolerances, purpose, owner, and review date. Malformed/adversarial fixtures are bounded and labeled.

The PartProbe source repository and its hosted CI are public. Before committing any new model or derivative, verify its classification and redistribution rights and record its provenance in the fixture manifest. Customer/shop/private/controlled models and private validation outputs remain outside Git history—including feature branches, issues, workflow artifacts, caches, logs, and support attachments. Repository visibility is not declassification or publication authority.

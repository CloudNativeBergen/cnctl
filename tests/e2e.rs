use cnctl::client::TrpcClient;
use cnctl::commands::{proposals, sponsors};
use cnctl::config::{self, Config};
use cnctl::template;
use tempfile::TempDir;
use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn proposal_json() -> serde_json::Value {
    serde_json::json!({
        "result": {
            "data": [
                {
                    "_id": "talk-abc",
                    "title": "Scaling Kubernetes in Production",
                    "status": "submitted",
                    "format": "presentation_25",
                    "level": "intermediate",
                    "language": "en",
                    "speakers": [
                        {"_id": "sp-1", "name": "Alice Johnson", "email": "alice@example.com"}
                    ],
                    "topics": [{"title": "Kubernetes"}, {"title": "DevOps"}],
                    "reviews": [
                        {"score": {"content": 7.0, "relevance": 8.0, "speaker": 6.0}, "comment": "Solid proposal", "reviewer": {"name": "Bob"}}
                    ]
                },
                {
                    "_id": "talk-def",
                    "title": "Intro to GitOps",
                    "status": "accepted",
                    "format": "lightning_10",
                    "speakers": [],
                    "topics": [],
                    "reviews": []
                }
            ]
        }
    })
}

fn single_proposal_json() -> serde_json::Value {
    serde_json::json!({
        "result": {
            "data": {
                "_id": "talk-abc",
                "title": "Scaling Kubernetes in Production",
                "status": "submitted",
                "format": "presentation_25",
                "level": "intermediate",
                "language": "en",
                "outline": "1. Introduction\n2. Architecture\n3. Demo",
                "speakers": [
                    {"_id": "sp-1", "name": "Alice Johnson", "email": "alice@example.com"}
                ],
                "topics": [{"title": "Kubernetes"}],
                "reviews": [
                    {"score": {"content": 7.0, "relevance": 8.0, "speaker": 6.0}, "comment": "Solid proposal", "reviewer": {"name": "Bob"}},
                    {"score": {"content": 9.0, "relevance": 10.0, "speaker": 9.0}, "comment": "Must accept!", "reviewer": {"name": "Carol"}}
                ]
            }
        }
    })
}

fn sponsor_list_json() -> serde_json::Value {
    serde_json::json!({
        "result": {
            "data": [
                {
                    "_id": "sfc-111",
                    "status": "closed-won",
                    "contractStatus": "contract-signed",
                    "invoiceStatus": "paid",
                    "sponsor": {"_id": "sp-a", "name": "Acme Corp", "website": "https://acme.com"},
                    "tier": {"_id": "tier-1", "title": "Gold"},
                    "assignedTo": {"_id": "org-1", "name": "Hans"},
                    "contactPersons": [
                        {"name": "Jane Doe", "email": "jane@acme.com", "role": "CTO", "isPrimary": true}
                    ],
                    "billing": {"email": "billing@acme.com", "reference": "PO-2025-001"},
                    "contractValue": 50000.0,
                    "contractCurrency": "NOK",
                    "notes": "Long-time partner",
                    "tags": ["returning", "premium"]
                },
                {
                    "_id": "sfc-222",
                    "status": "prospect",
                    "sponsor": {"_id": "sp-b", "name": "StartupCo"},
                    "contactPersons": [],
                    "tags": []
                }
            ]
        }
    })
}

// ─── Proposal e2e tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn proposals_list_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/proposal.admin.list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(proposal_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = proposals::fetch_all(&client).await;
    assert!(result.is_ok(), "fetch_all failed: {result:?}");
    let proposals = result.unwrap();
    assert_eq!(proposals.len(), 2);
    assert_eq!(proposals[0].title, "Scaling Kubernetes in Production");
}

#[tokio::test]
async fn proposals_list_empty_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/proposal.admin.list"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": {"data": []}})),
        )
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = proposals::fetch_all(&client).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn proposals_get_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/proposal.admin.getById"))
        .and(query_param_contains("input", "talk-abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(single_proposal_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = proposals::fetch_one(&client, "talk-abc").await;
    assert!(result.is_ok(), "fetch_one failed: {result:?}");
}

#[tokio::test]
async fn proposals_get_unauthorized_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/proposal.admin.getById"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {"message": "UNAUTHORIZED"}
        })))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "bad-token");
    let result = proposals::fetch_one(&client, "talk-abc").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UNAUTHORIZED"),
        "Expected UNAUTHORIZED, got: {err}"
    );
}

// ─── Sponsor e2e tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn sponsors_list_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.crm.list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sponsor_list_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = sponsors::fetch_all(&client).await;
    assert!(result.is_ok(), "fetch_all failed: {result:?}");
}

#[tokio::test]
async fn sponsors_list_empty_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.crm.list"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": {"data": []}})),
        )
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = sponsors::fetch_all(&client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn sponsors_get_existing_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.crm.list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sponsor_list_json()))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let sponsors = sponsors::fetch_all(&client).await;
    assert!(sponsors.is_ok(), "fetch_all failed: {sponsors:?}");
    let found = sponsors.unwrap().iter().any(|s| s.id == "sfc-111");
    assert!(found, "Expected to find sfc-111");
}

#[tokio::test]
async fn sponsors_get_not_found_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.crm.list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sponsor_list_json()))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let sponsors = sponsors::fetch_all(&client).await.unwrap();
    let found = sponsors.iter().any(|s| s.id == "nonexistent");
    assert!(!found, "Should not find nonexistent sponsor");
}

// ─── Review e2e tests ────────────────────────────────────────────────────────

fn review_response_json() -> serde_json::Value {
    serde_json::json!({
        "result": {
            "data": {
                "_id": "review-123",
                "_rev": "abc",
                "_createdAt": "2026-04-04T20:00:00Z",
                "_updatedAt": "2026-04-04T20:00:00Z",
                "score": {"content": 4, "relevance": 3, "speaker": 5},
                "comment": "Excellent proposal"
            }
        }
    })
}

#[tokio::test]
async fn submit_review_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/trpc/proposal.admin.submitReview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(review_response_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let input = cnctl::types::ReviewInput {
        id: "talk-abc".to_string(),
        comment: "Excellent proposal".to_string(),
        score: cnctl::types::ReviewScore {
            content: 4.0,
            relevance: 3.0,
            speaker: 5.0,
        },
    };
    let result = proposals::submit_review(&client, &input).await;
    assert!(result.is_ok(), "submit_review failed: {result:?}");
}

#[tokio::test]
async fn submit_review_unauthorized_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/trpc/proposal.admin.submitReview"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {"message": "UNAUTHORIZED"}
        })))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "bad-token");
    let input = cnctl::types::ReviewInput {
        id: "talk-abc".to_string(),
        comment: "Test".to_string(),
        score: cnctl::types::ReviewScore {
            content: 3.0,
            relevance: 3.0,
            speaker: 3.0,
        },
    };
    let result = proposals::submit_review(&client, &input).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UNAUTHORIZED"),
        "Expected UNAUTHORIZED, got: {err}"
    );
}

// ─── Config + status/logout e2e tests ────────────────────────────────────────

#[test]
fn config_roundtrip_e2e() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    let cfg = Config {
        api_url: "https://2026.cloudnativedays.no".to_string(),
        token: "jwt-xyz".to_string(),
        conference_id: "conf-2026".to_string(),
        conference_title: "2026.cloudnativedays.no".to_string(),
        name: None,
    };

    // Save → load → verify roundtrip
    config::save_to(&cfg, &path).unwrap();
    let loaded = config::load_from(&path).unwrap();
    assert_eq!(cfg, loaded);

    // Delete → verify gone
    assert!(config::delete_at(&path).unwrap());
    assert!(!path.exists());

    // Delete again → returns false
    assert!(!config::delete_at(&path).unwrap());
}

// ─── Full pipeline: login config → query → display ──────────────────────────

#[tokio::test]
async fn full_pipeline_proposals_e2e() {
    // Simulates: login writes config → proposals list reads config → queries API → renders
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/proposal.admin.list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(proposal_json()))
        .expect(1)
        .mount(&server)
        .await;

    // Step 1: Simulate login by writing config
    let cfg = Config {
        api_url: server.uri(),
        token: "test-jwt".to_string(),
        conference_id: "conf-2026".to_string(),
        conference_title: "Cloud Native Days 2026".to_string(),
        name: None,
    };
    config::save_to(&cfg, &config_path).unwrap();

    // Step 2: Load config and create client
    let loaded = config::load_from(&config_path).unwrap();
    let client = TrpcClient::from_config(&loaded);

    // Step 3: Run the actual command logic
    let result = proposals::fetch_all(&client).await;
    assert!(result.is_ok(), "Full pipeline failed: {result:?}");
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn full_pipeline_sponsors_e2e() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.crm.list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sponsor_list_json()))
        .expect(1)
        .mount(&server)
        .await;

    // Simulate login
    let cfg = Config {
        api_url: server.uri(),
        token: "test-jwt".to_string(),
        conference_id: "conf-2026".to_string(),
        conference_title: "Cloud Native Days 2026".to_string(),
        name: None,
    };
    config::save_to(&cfg, &config_path).unwrap();

    // Load and run
    let loaded = config::load_from(&config_path).unwrap();
    let client = TrpcClient::from_config(&loaded);
    let result = sponsors::fetch_all(&client).await;
    assert!(result.is_ok(), "Full pipeline failed: {result:?}");
}

#[tokio::test]
async fn server_error_propagates_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/proposal.admin.list"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = proposals::fetch_all(&client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"), "Expected 500 in error, got: {err}");
}

// ─── Email template e2e tests ────────────────────────────────────────────────

fn template_list_json() -> serde_json::Value {
    serde_json::json!({
        "result": {
            "data": {
                "templates": [
                    {
                        "_id": "tmpl-1",
                        "title": "Cold Outreach (English)",
                        "slug": {"current": "cold-outreach-en"},
                        "category": "cold-outreach",
                        "language": "en",
                        "subject": "Partnership with {{{CONFERENCE_TITLE}}}",
                        "bodyMarkdown": "Dear {{{CONTACT_NAMES}}},\n\nWe'd love to have **{{{SPONSOR_NAME}}}** as a sponsor for {{{CONFERENCE_TITLE}}}.\n\nBest regards,\n{{{SENDER_NAME}}}",
                        "description": "Initial outreach to new sponsors",
                        "isDefault": true,
                        "sortOrder": 1
                    },
                    {
                        "_id": "tmpl-2",
                        "title": "Follow-up (Norwegian)",
                        "slug": {"current": "follow-up-no"},
                        "category": "follow-up",
                        "language": "no",
                        "subject": "Oppfølging - {{{CONFERENCE_TITLE}}}",
                        "bodyMarkdown": "Hei {{{CONTACT_NAMES}}},\n\nVi følger opp vår henvendelse.",
                        "sortOrder": 2
                    }
                ],
                "variables": {
                    "SPONSOR_NAME": "Acme Corp",
                    "CONTACT_NAMES": "Jane Doe",
                    "CONFERENCE_TITLE": "Cloud Native Days 2026",
                    "SENDER_NAME": "Hans"
                },
                "recipients": [
                    {"name": "Jane Doe", "email": "jane@acme.com"}
                ],
                "sponsorName": "Acme Corp"
            }
        }
    })
}

fn send_email_response_json() -> serde_json::Value {
    serde_json::json!({
        "result": {
            "data": {
                "success": true,
                "emailId": "email-abc-123",
                "recipientCount": 1
            }
        }
    })
}

#[tokio::test]
async fn email_fetch_templates_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.emailTemplates.listForSponsor"))
        .and(query_param_contains("input", "sfc-111"))
        .respond_with(ResponseTemplate::new(200).set_body_json(template_list_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let result = sponsors::email::fetch_templates(&client, "sfc-111").await;
    assert!(result.is_ok(), "fetch_templates failed: {result:?}");

    let resp = result.unwrap();
    assert_eq!(resp.templates.len(), 2);
    assert_eq!(resp.templates[0].title, "Cold Outreach (English)");
    assert_eq!(resp.templates[0].slug.current, "cold-outreach-en");
    assert_eq!(resp.variables.get("SPONSOR_NAME").unwrap(), "Acme Corp");
    assert_eq!(resp.sponsor_name.as_deref(), Some("Acme Corp"));
    assert_eq!(resp.recipients.len(), 1);
    assert_eq!(resp.recipients[0].email, "jane@acme.com");
}

#[tokio::test]
async fn email_template_variable_substitution_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.emailTemplates.listForSponsor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(template_list_json()))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let resp = sponsors::email::fetch_templates(&client, "sfc-111")
        .await
        .unwrap();

    let tmpl = &resp.templates[0];
    let subject = template::substitute_variables(&tmpl.subject, &resp.variables);
    let body = template::substitute_variables(
        tmpl.body_markdown.as_deref().unwrap_or(""),
        &resp.variables,
    );

    assert_eq!(subject, "Partnership with Cloud Native Days 2026");
    assert!(body.contains("Acme Corp"));
    assert!(body.contains("Jane Doe"));
    assert!(body.contains("Hans"));

    // No unresolved variables
    assert!(template::find_unresolved_variables(&subject).is_empty());
    assert!(template::find_unresolved_variables(&body).is_empty());
}

#[tokio::test]
async fn email_send_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/trpc/sponsor.crm.sendEmailBySfc"))
        .and(body_string_contains("sfc-111"))
        .respond_with(ResponseTemplate::new(200).set_body_json(send_email_response_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let input = serde_json::json!({
        "sponsorForConferenceId": "sfc-111",
        "subject": "Hello from tests",
        "body": "This is a test email body.",
    });
    let result: cnctl::types::SendEmailResponse = client
        .mutate("sponsor.crm.sendEmailBySfc", &input)
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.email_id.as_deref(), Some("email-abc-123"));
    assert_eq!(result.recipient_count, Some(1));
}

#[tokio::test]
async fn email_send_unauthorized_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/trpc/sponsor.crm.sendEmailBySfc"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {"message": "UNAUTHORIZED"}
        })))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "bad-token");
    let input = serde_json::json!({
        "sponsorForConferenceId": "sfc-111",
        "subject": "Test",
        "body": "Test body",
    });
    let result: Result<cnctl::types::SendEmailResponse, _> = client
        .mutate("sponsor.crm.sendEmailBySfc", &input)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UNAUTHORIZED"),
        "Expected UNAUTHORIZED, got: {err}"
    );
}

#[tokio::test]
async fn email_fetch_templates_empty_e2e() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/trpc/sponsor.emailTemplates.listForSponsor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "data": {
                    "templates": [],
                    "variables": {},
                    "recipients": []
                }
            }
        })))
        .mount(&server)
        .await;

    let client = TrpcClient::new(&server.uri(), "test-token");
    let resp = sponsors::email::fetch_templates(&client, "sfc-999")
        .await
        .unwrap();

    assert!(resp.templates.is_empty());
    assert!(resp.variables.is_empty());
    assert!(resp.recipients.is_empty());
}

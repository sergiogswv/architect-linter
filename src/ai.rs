use serde::{Deserialize, Serialize};
use crate::config::AIConfig;

// Estructuras para el mapeo de la respuesta de la IA
#[derive(Deserialize, Serialize, Debug)]
pub struct AISuggestionResponse {
    pub pattern: String,
    pub suggested_max_lines: usize,
    pub rules: Vec<SuggestedRule>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SuggestedRule {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// Función exclusiva para el Linter: Sugiere la arquitectura inicial
pub fn sugerir_arquitectura_inicial(
    context: crate::discovery::ProjectContext,
    ai_config: AIConfig,
) -> anyhow::Result<AISuggestionResponse> {
    let prompt = format!(
        "Eres un Arquitecto de Software Senior. Analiza este proyecto {framework} con las siguientes dependencias: {deps:?}
        y esta estructura de archivos: {files:?}.

        TAREA:
        Identifica el patrón (Hexagonal, Clean o MVC) y sugiere reglas de importaciones prohibidas basándote en las mejores prácticas.

        RESPONDE EXCLUSIVAMENTE EN FORMATO JSON con esta estructura:
        {{
          \"pattern\": \"Nombre del patrón\",
          \"suggested_max_lines\": 60,
          \"rules\": [
            {{ \"from\": \"patrón_origen\", \"to\": \"patrón_prohibido\", \"reason\": \"explicación corta\" }}
          ]
        }}",
        framework = context.framework,
        deps = context.dependencies,
        files = context.folder_structure
    );

    let respuesta = consultar_claude(prompt, ai_config)?;

    // Limpieza de la respuesta para asegurar que solo procesamos el JSON
    let json_start = respuesta
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("No se encontró JSON en la respuesta"))?;
    let json_end = respuesta.rfind('}').unwrap_or(respuesta.len() - 1) + 1;
    let clean_json = &respuesta[json_start..json_end];

    let sugerencia: AISuggestionResponse = serde_json::from_str(clean_json)?;
    Ok(sugerencia)
}

/// Consulta la API de Claude (Anthropic) con el prompt dado
fn consultar_claude(prompt: String, ai_config: AIConfig) -> anyhow::Result<String> {
    let url = format!("{}/v1/messages", ai_config.api_url.trim_end_matches('/'));

    // Crear el runtime de tokio para la llamada asíncrona
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "model": ai_config.model,
            "max_tokens": 1024,
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        });

        let response = client
            .post(&url)
            .header("x-api-key", &ai_config.api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Error en la API ({}): {}",
                status,
                response_text
            ));
        }

        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        // Extraer el texto de la respuesta
        let content = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No se pudo extraer el texto de la respuesta"))?;

        Ok(content.to_string())
    })
}

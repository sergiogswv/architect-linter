use crate::config::{AIConfig, AIProvider};
use serde::{Deserialize, Serialize};

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

/// Obtiene la lista de modelos disponibles para el proveedor configurado
pub fn obtener_modelos_disponibles(
    provider: &AIProvider,
    api_url: &str,
    api_key: &str,
) -> anyhow::Result<Vec<String>> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let client = reqwest::Client::new();
        let url = api_url.trim_end_matches('/');

        match provider {
            AIProvider::Claude => {
                let response = client
                    .get(&format!("{}/v1/models", url))
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .send()
                    .await?;

                let json: serde_json::Value = response.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Respuesta de Claude inválida"))?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            AIProvider::Gemini => {
                let response = client
                    .get(&format!("{}/v1beta/models?key={}", url, api_key))
                    .send()
                    .await?;

                let json: serde_json::Value = response.json().await?;
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Respuesta de Gemini inválida"))?
                    .iter()
                    .filter_map(|m| {
                        m["name"]
                            .as_str()
                            .map(|s| s.trim_start_matches("models/").to_string())
                    })
                    .collect();
                Ok(models)
            }
            AIProvider::OpenAI
            | AIProvider::Groq
            | AIProvider::Ollama
            | AIProvider::Kimi
            | AIProvider::DeepSeek => {
                let mut request = client.get(&format!("{}/models", url));
                if !api_key.is_empty() {
                    request = request.header("authorization", format!("Bearer {}", api_key));
                }

                let response = request.send().await?;
                let json: serde_json::Value = response.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Respuesta de API compatible inválida"))?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
        }
    })
}

/// Función para consultar la IA seleccionada de forma genérica
pub fn consultar_ia(prompt: String, ai_config: AIConfig) -> anyhow::Result<String> {
    match ai_config.provider {
        AIProvider::Claude => consultar_claude(prompt, ai_config),
        AIProvider::Gemini => consultar_gemini(prompt, ai_config),
        AIProvider::OpenAI | AIProvider::Groq | AIProvider::Ollama | AIProvider::Kimi => {
            consultar_openai_compatible(prompt, ai_config)
        }
        AIProvider::DeepSeek => consultar_openai_compatible(prompt, ai_config),
    }
}

/// Orquestador que intenta consultar varias IAs en orden hasta que una funcione
pub fn consultar_ia_con_fallback(prompt: String, configs: &[AIConfig]) -> anyhow::Result<String> {
    if configs.is_empty() {
        return Err(anyhow::anyhow!("No hay configuraciones de IA disponibles. Ejecuta el linter sin architect.json para configurar una."));
    }

    let mut last_error = anyhow::anyhow!("Error desconocido");

    for (i, config) in configs.iter().enumerate() {
        if i > 0 {
            println!(
                "\n⚠️  El modelo '{}' falló. Intentando con el siguiente configurado: '{}'...",
                configs[i - 1].name,
                config.name
            );
        }

        match consultar_ia(prompt.clone(), config.clone()) {
            Ok(res) => {
                if i > 0 {
                    println!("✅ El modelo '{}' respondió correctamente.\n", config.name);
                }
                return Ok(res);
            }
            Err(e) => {
                println!("❌ Error en '{}': {}", config.name, e);
                last_error = e;
            }
        }
    }

    Err(anyhow::anyhow!(
        "❌ Todos los modelos configurados fallaron. Último error: {}",
        last_error
    ))
}

/// Función exclusiva para el Linter: Sugiere la arquitectura inicial
pub fn sugerir_arquitectura_inicial(
    context: crate::discovery::ProjectContext,
    ai_configs: Vec<AIConfig>,
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

    // Obtener respuesta con fallback
    let response_text = consultar_ia_con_fallback(prompt, &ai_configs)?;

    // Limpiar el JSON de posibles textos adicionales de la IA
    let json_start = response_text
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("No se encontró JSON en la respuesta"))?;
    let json_end = response_text
        .rfind('}')
        .ok_or_else(|| anyhow::anyhow!("No se encontró JSON en la respuesta"))?;
    let clean_json = &response_text[json_start..=json_end];

    let suggestion: AISuggestionResponse = serde_json::from_str(clean_json)?;
    Ok(suggestion)
}

/// Consulta la API de Claude (Anthropic)
fn consultar_claude(prompt: String, ai_config: AIConfig) -> anyhow::Result<String> {
    let url = format!("{}/v1/messages", ai_config.api_url.trim_end_matches('/'));
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
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        procesar_respuesta(response).await
    })
}

/// Consulta la API de Gemini (Google)
fn consultar_gemini(prompt: String, ai_config: AIConfig) -> anyhow::Result<String> {
    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        ai_config.api_url.trim_end_matches('/'),
        ai_config.model,
        ai_config.api_key
    );
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Error Gemini ({}): {}",
                status,
                response_text
            ));
        }

        let json: serde_json::Value = serde_json::from_str(&response_text)?;
        let content = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No se pudo extraer texto de Gemini"))?;

        Ok(content.to_string())
    })
}

/// Consulta APIs compatibles con OpenAI (OpenAI, Groq, Ollama)
fn consultar_openai_compatible(prompt: String, ai_config: AIConfig) -> anyhow::Result<String> {
    let url = format!(
        "{}/chat/completions",
        ai_config.api_url.trim_end_matches('/')
    );
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": ai_config.model,
            "messages": [
                {"role": "system", "content": "Eres un Arquitecto de Software Senior."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.1
        });

        let mut request = client.post(&url).header("content-type", "application/json");

        if !ai_config.api_key.is_empty() {
            request = request.header("authorization", format!("Bearer {}", ai_config.api_key));
        }

        let response = request.json(&body).send().await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("Error API ({}): {}", status, response_text));
        }

        let json: serde_json::Value = serde_json::from_str(&response_text)?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No se pudo extraer texto de la respuesta"))?;

        Ok(content.to_string())
    })
}

async fn procesar_respuesta(response: reqwest::Response) -> anyhow::Result<String> {
    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("Error API ({}): {}", status, response_text));
    }

    let json: serde_json::Value = serde_json::from_str(&response_text)?;

    // Claude format
    if let Some(content) = json["content"][0]["text"].as_str() {
        return Ok(content.to_string());
    }

    Ok(response_text)
}

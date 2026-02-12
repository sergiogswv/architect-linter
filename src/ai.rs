use crate::config::{AIConfig, AIProvider};
use serde::{Deserialize, Serialize};

/// Extrae el primer objeto JSON válido de un texto, manejando correctamente las llaves anidadas
/// y eliminando marcadores de markdown (```json, ```, etc.)
fn extract_json_object(text: &str) -> Option<String> {
    // Primero, limpiar los marcadores de código markdown
    let cleaned_text = text
        .replace("```json", "")
        .replace("```", "")
        .trim()
        .to_string();

    let start = cleaned_text.find('{')?;
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut result = String::new();

    // Construir el JSON carácter por carácter
    for ch in cleaned_text[start..].chars() {
        result.push(ch);

        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => brace_count += 1,
            '}' if !in_string => {
                brace_count -= 1;
                if brace_count == 0 {
                    // Encontramos el cierre del objeto JSON principal
                    return Some(result);
                }
            }
            _ => {}
        }
    }

    None
}

// Helper para deserializar campos que pueden venir como String o Array<String>
fn deserialize_string_or_array<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrArray;

    impl<'de> Visitor<'de> for StringOrArray {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or an array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<String, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            // Si es un array, tomamos el primer elemento
            if let Some(first) = seq.next_element::<String>()? {
                Ok(first)
            } else {
                Err(de::Error::custom("array vacío"))
            }
        }
    }

    deserializer.deserialize_any(StringOrArray)
}

// Estructuras para el mapeo de la respuesta de la IA
#[derive(Deserialize, Serialize, Debug)]
pub struct AISuggestionResponse {
    #[serde(deserialize_with = "deserialize_string_or_array")]
    pub pattern: String,
    pub suggested_max_lines: usize,
    pub rules: Vec<SuggestedRule>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SuggestedRule {
    #[serde(deserialize_with = "deserialize_string_or_array")]
    pub from: String,
    #[serde(deserialize_with = "deserialize_string_or_array")]
    pub to: String,
    #[serde(deserialize_with = "deserialize_string_or_array")]
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
        Identifica el patrón arquitectónico (Hexagonal, Clean, MVC o Ninguno) y sugiere entre 2 y 5 reglas de importaciones prohibidas basándote en las mejores prácticas.

        PRINCIPIOS A CONSIDERAR:
        1. **DRY (Don't Repeat Yourself)**: Detecta patrones de código duplicado, funciones repetitivas, o lógica que debería ser abstraída.
           - Identifica módulos que podrían estar repitiendo lógica similar
           - Sugiere reglas que promuevan la reutilización de código
           - Detecta dependencias que indiquen duplicación de responsabilidades
        2. **Separación de Responsabilidades**: Cada módulo debe tener una única responsabilidad clara
        3. **Inversión de Dependencias**: Las capas de alto nivel no deben depender de las de bajo nivel

        INSTRUCCIONES IMPORTANTES:
        1. Responde ÚNICAMENTE con JSON válido, sin texto adicional antes o después
        2. Asegúrate de cerrar todas las llaves y corchetes correctamente
        3. Limita las reglas a máximo 3 para evitar respuestas muy largas
        4. Usa comillas dobles para todos los strings
        5. Cada razón debe ser concisa (máximo 15 palabras)

        FORMATO JSON REQUERIDO:
        {{
          \"pattern\": \"Hexagonal\",
          \"suggested_max_lines\": 60,
          \"rules\": [
            {{
              \"from\": \"src/presentation/**\",
              \"to\": \"src/infrastructure/**\",
              \"reason\": \"La capa de presentación no debe depender de infraestructura\"
            }}
          ]
        }}

        RESPUESTA (solo JSON):",
        framework = context.framework,
        deps = context.dependencies,
        files = context.folder_structure
    );

    // Obtener respuesta con fallback
    let response_text = consultar_ia_con_fallback(prompt, &ai_configs)?;

    // Extraer el JSON válido usando un contador de llaves
    let clean_json = match extract_json_object(&response_text) {
        Some(json) => json,
        None => {
            eprintln!("\n❌ No se encontró un JSON válido en la respuesta de la IA");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("📄 Respuesta completa recibida:");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("{}", response_text);
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            return Err(anyhow::anyhow!("No se encontró un JSON válido en la respuesta"));
        }
    };

    // Intentar parsear con mejor manejo de errores
    let suggestion: AISuggestionResponse = serde_json::from_str(&clean_json)
        .map_err(|e| {
            eprintln!("\n❌ Error parseando JSON de la IA:");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("{}", e);
            eprintln!("\n📄 JSON extraído:");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("{}", clean_json);
            eprintln!("\n📄 Respuesta completa de la IA:");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("{}", response_text);
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            anyhow::anyhow!("{}", e)
        })?;
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
            "max_tokens": 8192,
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
            "temperature": 0.1,
            "max_tokens": 8192
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

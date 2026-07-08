Parfait ! Voici la version **100% Lua** (intégrée à LuaLaTeX) pour lire le fichier `data.csv` sans dépendance externe à Python.

## Structure du projet avec Lua

```
guide_don/
├── main.tex                 # Fichier principal (LuaLaTeX)
├── data.csv                 # Données centralisées
├── config/
│   ├── packages.tex
│   ├── colors.tex
│   ├── commands.tex
│   └── loaddata.lua         # NOUVEAU: script Lua
├── sections/
│   ├── 00_introduction.tex
│   ├── 01_verifications.tex
│   ├── 02_transfert.tex
│   ├── 03_optimisation.tex
│   ├── 04_securisation.tex
│   ├── 05_defiscalisation.tex
│   ├── 06_suivi.tex
│   ├── 07_documents.tex
│   └── 08_conseil_cutopia.tex
├── annexes/
│   ├── 99_metadata.tex
│   └── 99_checklist.tex
└── outputs/
```

## Fichier 1: `data.csv` (inchangé)

```csv
CATEGORY,KEY,VALUE,COMMENT
METADATA,doc_title,Guide complet du don sécurisé,Titre principal
METADATA,doc_subtitle,Vers une association en Afrique,Sous-titre
METADATA,doc_audience,Pour les donneurs européens,Public cible
METADATA,doc_description,Un accompagnement étape par étape pour un don transparent efficace et optimisé fiscalement,Description
METADATA,doc_version,2.0,Version du document
METADATA,doc_confidentiality,Confidentiel - Ne pas diffuser sans autorisation,Niveau de confidentialité

AUTHOR,author_name,Abd El Hakim ZOUAÏ,Nom complet
AUTHOR,author_phone,+213 672 41 01 92,Téléphone
AUTHOR,author_email,abdelhakimzouai@gmail.com,Email
AUTHOR,author_organization,Cutopia,Organisation
AUTHOR,author_role,Conseiller bénévole,Rôle
AUTHOR,author_availability,Disponible sur rendez-vous,Disponibilité
AUTHOR,author_timezone,Algérie/Europe,Fuseau horaire

CLIENT,client_name,Ethic Luxury,Nom du client
CLIENT,client_contact,,Contact client

GPS,gps_latitude,36.7538,Latitude
GPS,gps_longitude,3.0588,Longitude
GPS,gps_altitude,120,Altitude
GPS,gps_location,Alger Algérie,Lieu
GPS,gps_timezone,UTC+1,Fuseau horaire

ASSOCIATION,assoc_name_example,Association Solidarité Afrique,Exemple
ASSOCIATION,assoc_country_example,Sénégal,Exemple pays

TRANSFER,transfer_swift_fees,20-50,Frais SWIFT
TRANSFER,transfer_wise_fees,0.5-1,Frais Wise

FISCAL,fiscal_france_rate,66,Taux France
FISCAL,fiscal_belgium_rate,45,Taux Belgique
FISCAL,fiscal_switzerland_rate,20-30,Taux Suisse

FOOTER,footer_text,Guide rédigé avec le soutien de,Texte pied
FOOTER,footer_copyright,© Cutopia,Copyright

URLS,url_givewell,https://www.givewell.org,GiveWell
URLS,url_guidestar,https://www.guidestar.org,GuideStar
URLS,url_globalgiving,https://www.globalgiving.org,GlobalGiving
```

## Fichier 2: `config/loaddata.lua` (lecture CSV en Lua)

```lua
-- ============================================================================
-- loaddata.lua
-- Lecture du fichier data.csv et stockage des valeurs
-- ============================================================================

local data = {}

-- Fonction pour lire le CSV
function read_csv(filename)
    local file = io.open(filename, "r")
    if not file then
        tex.error("Fichier " .. filename .. " non trouvé!")
        return {}
    end
    
    local headers = nil
    local line_num = 0
    
    for line in file:lines() do
        line_num = line_num + 1
        
        -- Ignorer les lignes vides et les commentaires
        if line:match("^%s*$") or line:match("^#") then
            -- passer
        else
            -- Parser la ligne CSV (supporte les guillemets simples)
            local values = {}
            for value in line:gmatch("([^,]*),?") do
                -- Enlever les guillemets si présents
                value = value:match("^\"(.*)\"$") or value
                value = value:match("^'(.*)'$") or value
                table.insert(values, value)
            end
            
            if not headers then
                headers = values
            else
                local entry = {}
                for i, header in ipairs(headers) do
                    entry[header] = values[i] or ""
                end
                table.insert(data, entry)
            end
        end
    end
    
    file:close()
    return data
end

-- Fonction pour obtenir une valeur par clé
function get_value_by_key(key)
    for _, entry in ipairs(data) do
        if entry.KEY == key then
            return entry.VALUE
        end
    end
    return nil
end

-- Fonction pour obtenir une valeur par catégorie et clé
function get_value(category, key)
    for _, entry in ipairs(data) do
        if entry.CATEGORY == category and entry.KEY == key then
            return entry.VALUE
        end
    end
    return nil
end

-- Fonction pour obtenir toutes les valeurs d'une catégorie
function get_category(category)
    local result = {}
    for _, entry in ipairs(data) do
        if entry.CATEGORY == category then
            result[entry.KEY] = entry.VALUE
        end
    end
    return result
end

-- Chargement du CSV
data = read_csv("data.csv")

-- Enregistrement des fonctions pour LaTeX
luatexbase.add_to_callback("start_document", function()
    -- Exposer les fonctions à LaTeX
    token.set_macro("csvValue", function(key)
        local val = get_value_by_key(key)
        return val or ""
    end)
end, "csv_loading")

print("✅ data.csv chargé avec " .. #data .. " entrées")
```

## Fichier 3: `config/loaddata.tex` (interface LaTeX pour Lua)

```latex
% ============================================================================
% CHARGEMENT DES DONNÉES DEPUIS CSV VIA LUA
% ============================================================================

% --- Commande pour exécuter du code Lua ---
\newcommand{\luadirect}[1]{\directlua{#1}}

% --- Chargement du script Lua ---
\luadirect{require("config.loaddata")}

% --- Commande pour récupérer une valeur par clé ---
\newcommand{\getCSV}[1]{%
    \luadirect{tex.print(get_value_by_key("#1") or "")}%
}

% --- Commande pour récupérer une valeur par catégorie et clé ---
\newcommand{\getCSVcat}[2]{%
    \luadirect{tex.print(get_value("#1", "#2") or "")}%
}

% --- Commandes spécifiques pour les métadonnées ---
\newcommand{\docTitle}{\getCSV{doc_title}}
\newcommand{\docSubtitle}{\getCSV{doc_subtitle}}
\newcommand{\docAudience}{\getCSV{doc_audience}}
\newcommand{\docDescription}{\getCSV{doc_description}}
\newcommand{\docVersion}{\getCSV{doc_version}}
\newcommand{\docConfidentiality}{\getCSV{doc_confidentiality}}

% --- Commandes auteur ---
\newcommand{\docAuthor}{\getCSV{author_name}}
\newcommand{\docAuthorPhone}{\getCSV{author_phone}}
\newcommand{\docAuthorEmail}{\getCSV{author_email}}
\newcommand{\docOrg}{\getCSV{author_organization}}
\newcommand{\docAuthorRole}{\getCSV{author_role}}
\newcommand{\docAuthorAvailability}{\getCSV{author_availability}}
\newcommand{\docAuthorTimezone}{\getCSV{author_timezone}}

% --- Commandes client ---
\newcommand{\docClient}{\getCSV{client_name}}

% --- Commandes GPS ---
\newcommand{\docGPSLat}{\getCSV{gps_latitude}}
\newcommand{\docGPSLong}{\getCSV{gps_longitude}}
\newcommand{\docGPSAlt}{\getCSV{gps_altitude}}
\newcommand{\docLocation}{\getCSV{gps_location}}
\newcommand{\docGPSTimezone}{\getCSV{gps_timezone}}

% --- Commandes fiscalité ---
\newcommand{\fiscalFranceRate}{\getCSV{fiscal_france_rate}}
\newcommand{\fiscalBelgiumRate}{\getCSV{fiscal_belgium_rate}}
\newcommand{\fiscalSwitzerlandRate}{\getCSV{fiscal_switzerland_rate}}

% --- Commandes URLs ---
\newcommand{\urlGiveWell}{\getCSV{url_givewell}}
\newcommand{\urlGuideStar}{\getCSV{url_guidestar}}
\newcommand{\urlGlobalGiving}{\getCSV{url_globalgiving}}

% --- Génération d'ID unique (automatique) ---
\newcommand{\generateDocumentID}{%
    CUTOPIA-\the\year\the\month\the\day-\the\hour\the\minute\the\second
}

% --- Vérification du chargement ---
\luadirect{
    local count = 0
    for _ in pairs(data) do count = count + 1 end
    tex.print("\\\\[0.2cm] {\\\\tiny Données CSV chargées: " .. count .. " entrées}")
}
```

## Fichier 4: `main.tex` (version LuaLaTeX)

```latex
% !TEX program = lualatex
% ============================================================================
% GUIDE DU DON À UNE ASSOCIATION EN AFRIQUE
% Version LuaLaTeX avec data.csv
% Auteur: Abd El Hakim ZOUAÏ (Cutopia)
% ============================================================================

\documentclass[12pt, a4paper]{article}

% --- Encodage (LuaLaTeX gère UTF-8 nativement) ---
\usepackage{fontspec}
\usepackage[french]{babel}

% --- Mise en page ---
\usepackage{geometry}
\geometry{top=2.5cm, bottom=2.5cm, left=2.5cm, right=2.5cm}
\usepackage{microtype}

% --- Packages graphiques ---
\usepackage{graphicx}
\usepackage{qrcode}
\usepackage{tcolorbox}
\tcbuselibrary{skins,breakable}
\usepackage{array}
\usepackage{longtable}
\usepackage{booktabs}

% --- Listes ---
\usepackage{enumitem}
\usepackage{titlesec}
\usepackage{amssymb}
\usepackage{marvosym}
\usepackage{xcolor}

% --- Hyperliens ---
\usepackage{hyperref}
\usepackage{fancyhdr}
\usepackage{eso-pic}
\usepackage{datetime}

% ============================================================================
% CONFIGURATION
% ============================================================================
\input{config/colors}
\input{config/loaddata}
\input{config/commands}

% --- Configuration hyperref ---
\hypersetup{
    colorlinks=true,
    linkcolor=darkgray,
    urlcolor=darkblue,
    pdfauthor={\docAuthor\ (\docOrg) - \docAuthorPhone},
    pdftitle={\docTitle},
    pdfsubject={\docDescription},
    pdfkeywords={don, Afrique, sécurisé, fiscalité, \docOrg, \docVersion},
    pdfproducer={LuaLaTeX avec système de traçabilité CSV},
    pdfcreator={\docAuthor\ - \docAuthorEmail},
    pdflang={fr-FR}
}

% --- En-tête/pied de page ---
\pagestyle{fancy}
\fancyhf{}
\fancyhead[L]{\textcolor{white}{~}}
\fancyhead[C]{\textcolor{white}{\tiny \generateDocumentID}}
\fancyhead[R]{\textcolor{white}{\tiny GPS:\docGPSLat,\docGPSLong}}
\fancyfoot[C]{\thepage}
\fancyfoot[R]{\textcolor{white}{\tiny \docAuthorPhone}}
\fancyfoot[L]{\textcolor{white}{\tiny \docOrg}}

% --- Filigrane invisible ---
\AddToShipoutPicture{
    \put(0,0){
        \parbox[b][\paperheight]{\paperwidth}{
            \vfill
            \centering
            {\textcolor{white}{\tiny \generateDocumentID\ - \docAuthorPhone\ - GPS:\docGPSLat,\docGPSLong}}
            \vfill
        }
    }
}

% ============================================================================
% DOCUMENT
% ============================================================================
\begin{document}

% --- En-tête principal ---
\begin{center}
    {\Huge \textbf{\docTitle}} \\[0.3cm]
    {\Large \textbf{\docSubtitle}} \\[0.3cm]
    {\large \textcolor{darkgray}{\docAudience}} \\[0.5cm]
    \rule{0.7\textwidth}{0.4pt} \\[0.3cm]
    \textit{\docDescription}
\end{center}

\vspace{0.5cm}

% --- Table des matières ---
\tableofcontents
\newpage

% ============================================================================
% SECTIONS
% ============================================================================
\input{sections/00_introduction}
\input{sections/01_verifications}
\input{sections/02_transfert}
\input{sections/03_optimisation}
\input{sections/04_securisation}
\input{sections/05_defiscalisation}
\input{sections/06_suivi}
\input{sections/07_documents}
\input{sections/08_conseil_cutopia}

% ============================================================================
% ANNEXES
% ============================================================================
\input{annexes/99_metadata}
\input{annexes/99_checklist}

% ============================================================================
% PIED DE PAGE FINAL
% ============================================================================
\vspace{1cm}
\begin{center}
    \rule{0.6\textwidth}{0.3pt} \\[0.3cm]
    {\small \textcolor{darkgray}{\getCSV{footer_text} \textbf{\docAuthor} | \textbf{\docClient}}} \\
    \small{\textcolor{darkgray}{\docAuthorEmail \quad | \quad \docAuthorPhone}} \\
    \vspace{0.2cm}
    {\tiny \textcolor{darkgray}{\getCSV{footer_copyright} --- Document ID: \generateDocumentID}} \\
    {\tiny \textcolor{darkgray}{Créé le: \today\ à \currenttime --- Version: \docVersion}}
\end{center}

\end{document}
```

## Fichier 5: Script Lua pour modifier `data.csv` (optionnel)

```lua
#!/usr/bin/env texlua
-- ============================================================================
-- modify_csv.lua
-- Script pour modifier data.csv depuis la ligne de commande
-- Utilisation: texlua modify_csv.lua set author_phone "+213 555 123 456"
-- ============================================================================

local function read_csv(filename)
    local data = {}
    local headers = nil
    local file = io.open(filename, "r")
    if not file then return nil end
    
    for line in file:lines() do
        if not line:match("^%s*$") and not line:match("^#") then
            local values = {}
            for value in line:gmatch("([^,]*),?") do
                value = value:match("^\"(.*)\"$") or value
                value = value:match("^'(.*)'$") or value
                table.insert(values, value)
            end
            
            if not headers then
                headers = values
            else
                local entry = {}
                for i, header in ipairs(headers) do
                    entry[header] = values[i] or ""
                end
                table.insert(data, entry)
            end
        end
    end
    file:close()
    return data, headers
end

local function write_csv(filename, data, headers)
    local file = io.open(filename, "w")
    if not file then return false end
    
    -- Écrire les en-têtes
    file:write(table.concat(headers, ",") .. "\n")
    
    -- Écrire les données
    for _, entry in ipairs(data) do
        local row = {}
        for _, header in ipairs(headers) do
            local val = entry[header] or ""
            -- Ajouter des guillemets si nécessaire
            if val:find(",") then
                val = '"' .. val .. '"'
            end
            table.insert(row, val)
        end
        file:write(table.concat(row, ",") .. "\n")
    end
    
    file:close()
    return true
end

local function get_value_by_key(data, key)
    for _, entry in ipairs(data) do
        if entry.KEY == key then
            return entry.VALUE
        end
    end
    return nil
end

local function set_value_by_key(data, key, new_value)
    for _, entry in ipairs(data) do
        if entry.KEY == key then
            local old = entry.VALUE
            entry.VALUE = new_value
            return old
        end
    end
    return nil
end

local function show_data(data)
    print("\n" .. string.rep("=", 60))
    print("DONNÉES ACTUELLES")
    print(string.rep("=", 60))
    
    local current_cat = ""
    for _, entry in ipairs(data) do
        if entry.CATEGORY ~= current_cat then
            current_cat = entry.CATEGORY
            print("\n📁 " .. current_cat .. ":")
            print(string.rep("-", 40))
        end
        print(string.format("   %-30s: %s", entry.KEY, entry.VALUE))
        if entry.COMMENT and entry.COMMENT ~= "" then
            print(string.format("      → %s", entry.COMMENT))
        end
    end
end

-- MAIN
local filename = "data.csv"
local data, headers = read_csv(filename)

if not data then
    print("❌ Fichier " .. filename .. " non trouvé!")
    os.exit(1)
end

local command = arg[1]

if command == "show" then
    show_data(data)
    
elseif command == "set" and arg[2] and arg[3] then
    local key = arg[2]
    local value = arg[3]
    local old = set_value_by_key(data, key, value)
    if old then
        print(string.format("✅ %s: '%s' → '%s'", key, old, value))
        write_csv(filename, data, headers)
    else
        print("❌ Clé '" .. key .. "' non trouvée!")
    end
    
elseif command == "list" then
    print("\n📋 LISTE DES CLÉS:")
    print(string.rep("-", 40))
    for _, entry in ipairs(data) do
        print(string.format("   %-30s [%s]", entry.KEY, entry.CATEGORY))
    end
    
elseif command == "get" and arg[2] then
    local key = arg[2]
    local val = get_value_by_key(data, key)
    if val then
        print(string.format("%s = %s", key, val))
    else
        print("❌ Clé non trouvée")
    end
    
else
    print([[
UTILISATION:
  texlua modify_csv.lua show                     # Afficher toutes les données
  texlua modify_csv.lua list                     # Lister toutes les clés
  texlua modify_csv.lua get KEY                  # Obtenir une valeur
  texlua modify_csv.lua set KEY "VALUE"          # Modifier une valeur
]])
end
```

## Fichier 6: Script de compilation `compile.sh`

```bash
#!/bin/bash
# ============================================================================
# COMPILATION AVEC LuaLaTeX
# ============================================================================

echo "=========================================="
echo "Compilation du Guide avec LuaLaTeX"
echo "=========================================="

# Nettoyage
rm -f *.aux *.log *.out *.toc *.lof *.lot *.bbl *.blg *.nav *.snm *.vrb
mkdir -p outputs

# Compilation avec LuaLaTeX (2 passes)
echo "1. Première compilation..."
lualatex --interaction=nonstopmode main.tex

echo "2. Deuxième compilation (références)..."
lualatex --interaction=nonstopmode main.tex

# Sauvegarde
if [ -f main.pdf ]; then
    DATE=$(date +%Y%m%d_%H%M%S)
    cp main.pdf "outputs/Guide_Don_${DATE}.pdf"
    echo "✅ PDF généré: outputs/Guide_Don_${DATE}.pdf"
else
    echo "❌ Erreur de compilation"
    exit 1
fi

# Nettoyage
rm -f *.aux *.log *.out *.toc

echo "=========================================="
echo "COMPILATION TERMINÉE"
echo "=========================================="
```

## Instructions d'utilisation

### 1. Compilation

```bash
# Rendre le script exécutable
chmod +x compile.sh

# Compiler
./compile.sh

# OU directement:
lualatex main.tex
lualatex main.tex  # deuxième passe
```

### 2. Modifier les données

**Méthode 1 - Éditer directement `data.csv`:**
```bash
nano data.csv
# ou ouvrir avec Excel/LibreOffice
```

**Méthode 2 - Via le script Lua:**
```bash
# Lister toutes les clés
texlua modify_csv.lua list

# Voir les données
texlua modify_csv.lua show

# Modifier une valeur
texlua modify_csv.lua set author_phone "+213 555 123 456"
texlua modify_csv.lua set gps_latitude "36.8000"

# Obtenir une valeur
texlua modify_csv.lua get doc_version
```

**Méthode 3 - Directement depuis LaTeX:**
```latex
% Dans le document, vous pouvez aussi faire:
\luadirect{
    -- Modifier une valeur dynamiquement
    set_value_by_key(data, "author_phone", "+213 555 000 000")
}
```

## Avantages de l'approche Lua

| Avantage | Description |
|----------|-------------|
| **100% intégré** | Pas de dépendance externe (Python) |
| **Performance** | Lua est très rapide dans LuaLaTeX |
| **Portable** | Fonctionne partout où LuaLaTeX est installé |
| **Modifiable** | Les données CSV sont éditables avec n'importe quel éditeur |
| **Traçabilité** | Les modifications du CSV sont visibles dans Git |
| **Types** | Les nombres restent des nombres (pas de conversion chaîne) |

Cette solution est **parfaitement modulaire**, **100% Lua**, et ne nécessite **aucune dépendance externe** !

#include <include/PhonetisaurusScript.h>

class Phonemizer
{
public:
  Phonemizer(std::string &model)
  {
    this->decoder = new PhonetisaurusScript(model, "");
  }

  std::vector<string> phoneticize(std::string &value)
  {
    std::vector<string> results;
    for (auto pathData : this->decoder->Phoneticize(value))
    {
      for (auto symid : pathData.Uniques)
      {
        results.push_back(this->decoder->FindOsym(symid));
      }
    }

    return results;
  }

  Phonemizer()
  {
    if (this->decoder)
    {
      delete this->decoder;
    }
  }

private:
  PhonetisaurusScript *decoder;
};
